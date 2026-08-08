use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, Expr, ExprBinary, ExprLit, ExprIndex, Lit, BinOp};

/// Compiles an arithmetic expression into a BlindRoute `Circuit`.
///
/// Syntax:
/// ```ignore
/// circuit! {
///     inputs[0] * 0.7 + inputs[1] * 0.3
/// }
/// ```
///
/// Supports: `+`, `-`, `*`, parenthesized grouping, float/int literals,
/// `inputs[index]` references. Returns a `blindroute_core::Circuit`
/// ready for evaluation with any `FheScheme`.
#[proc_macro]
pub fn circuit(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    let mut counter = 0usize;
    let (tokens, output_node, num_inputs) = compile_expr(&expr, &mut counter);
    let expanded = quote! {
        {
            let mut __circuit = blindroute_core::Circuit::new(#num_inputs);
            #tokens
            __circuit.output(#output_node);
            __circuit
        }
    };
    TokenStream::from(expanded)
}

fn make_var(counter: &mut usize) -> proc_macro2::Ident {
    *counter += 1;
    format_ident!("__n{}", counter)
}

fn compile_expr(expr: &Expr, counter: &mut usize) -> (proc_macro2::TokenStream, proc_macro2::Ident, usize) {
    match expr {
        Expr::Binary(b) => compile_binary(b, counter),
        Expr::Paren(p) => compile_expr(&p.expr, counter),
        Expr::Index(i) => compile_index(i),
        Expr::Lit(l) => compile_lit(l, counter),
        Expr::Group(g) => compile_expr(&g.expr, counter),
        other => {
            let msg = format!("unsupported expression: {:?}", other);
            let v = make_var(counter);
            (quote! { compile_error!(#msg); let #v = 0usize; }, v, 0)
        }
    }
}

fn compile_binary(b: &ExprBinary, counter: &mut usize) -> (proc_macro2::TokenStream, proc_macro2::Ident, usize) {
    let (lc, lv, li) = compile_expr(&b.left, counter);
    let (rc, rv, ri) = compile_expr(&b.right, counter);
    let ni = li.max(ri);
    let result = make_var(counter);

    let method = match b.op {
        BinOp::Add(_) => quote! { add },
        BinOp::Sub(_) => quote! { sub },
        BinOp::Mul(_) => quote! { mul },
        _ => {
            let msg = format!("unsupported operator: {:?}", b.op);
            return (quote! { compile_error!(#msg); let #result = 0usize; }, result, 0);
        }
    };

    (quote! { #lc #rc let #result = __circuit.#method(#lv, #rv); }, result, ni)
}

fn compile_index(idx: &ExprIndex) -> (proc_macro2::TokenStream, proc_macro2::Ident, usize) {
    if let Expr::Path(path) = &*idx.expr {
        let path_str = path.path.segments.iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if path_str == "inputs" {
            if let Expr::Lit(lit) = &*idx.index {
                if let Lit::Int(i) = &lit.lit {
                    let val: usize = i.base10_parse().unwrap_or(0);
                    let name = format_ident!("__input_{}", val);
                    return (
                        quote! { let #name = __circuit.input_idx(#val); },
                        name,
                        val + 1,
                    );
                }
            }
        }
    }
    (quote! { compile_error!("only `inputs[number]` is supported"); },
     format_ident!("__bad"), 0)
}

fn compile_lit(lit: &ExprLit, counter: &mut usize) -> (proc_macro2::TokenStream, proc_macro2::Ident, usize) {
    let result = make_var(counter);
    match &lit.lit {
        Lit::Float(f) => {
            let val: f64 = f.base10_parse().unwrap_or(0.0);
            (quote! { let #result = __circuit.const_f64(#val); }, result, 0)
        }
        Lit::Int(i) => {
            let val: f64 = i.base10_parse().unwrap_or(0.0);
            (quote! { let #result = __circuit.const_f64(#val); }, result, 0)
        }
        _ => {
            (quote! { compile_error!("only numeric literals supported"); let #result = 0usize; }, result, 0)
        }
    }
}
