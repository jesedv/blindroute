use crate::shader::NTT_SHADER;
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: String,
}

impl GpuContext {
    pub async fn new() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;

        let info = format!("{:?}", adapter.get_info());
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await
            .ok()?;

        Some(GpuContext {
            device,
            queue,
            adapter_info: info,
        })
    }

    pub fn new_sync() -> Option<Self> {
        pollster::block_on(Self::new())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    n: u32,
    len: u32,
    q: u32,
    _pad: u32,
}

pub struct GpuNtt {
    ctx: Arc<GpuContext>,
    n: u32,
    q: u32,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl GpuNtt {
    pub fn new(ctx: &Arc<GpuContext>, n: u32, q: u32) -> Self {
        assert!(n.is_power_of_two());
        let device = &ctx.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ntt_shader"),
            source: wgpu::ShaderSource::Wgsl(NTT_SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ntt_bg_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ntt_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ntt_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "ntt_stage",
            compilation_options: Default::default(),
            cache: None,
        });

        GpuNtt {
            ctx: ctx.clone(),
            n,
            q,
            pipeline,
            bind_group_layout,
        }
    }

    pub fn run_forward_ntt(&self, data: &mut [u32]) {
        assert_eq!(data.len(), self.n as usize);

        let n = self.n;
        let q = self.q;
        let n_usize = n as usize;
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;

        self.bit_reverse_cpu(data);

        let data_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ntt_data"),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        });

        let mut len: u32 = 1;
        while len < n {
            let step = n / (2 * len);
            let wlen = blindroute_ntt::params::modpow(
                blindroute_ntt::params::nth_root(n as u64) as u64,
                step as u64,
                q as u64,
            ) as u32;

            let mut powers = vec![0u32; len as usize];
            let mut w = 1u64;
            for i in 0..len as usize {
                powers[i] = w as u32;
                w = (w * wlen as u64) % q as u64;
            }

            let wbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ntt_w"),
                contents: bytemuck::cast_slice(&powers),
                usage: wgpu::BufferUsages::STORAGE,
            });

            let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ntt_ubo"),
                contents: bytemuck::cast_slice(&[Uniforms {
                    n,
                    len,
                    q,
                    _pad: 0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ntt_bg"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: data_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wbuf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: ubuf.as_entire_binding(),
                    },
                ],
            });

            let workgroups = ((n / 2) as u32).div_ceil(256);

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ntt_stage_enc"),
            });
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("ntt_stage_pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.pipeline);
                cpass.set_bind_group(0, &bg, &[]);
                cpass.dispatch_workgroups(workgroups, 1, 1);
            }
            queue.submit(Some(encoder.finish()));

            len *= 2;
        }

        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ntt_readback"),
            size: (n_usize * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_buffer_to_buffer(&data_buf, 0, &staging, 0, (n_usize * 4) as u64);
        queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::Maintain::Wait);
        let view = slice.get_mapped_range();
        data.copy_from_slice(bytemuck::cast_slice(&view));
        drop(view);
        staging.unmap();
    }

    fn bit_reverse_cpu(&self, data: &mut [u32]) {
        let n = data.len();
        let mut j = 0usize;
        for i in 1..n {
            let mut bit = n >> 1;
            while j & bit != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j |= bit;
            if i < j {
                data.swap(i, j);
            }
        }
    }
}
