use soroban_wasmi as wasmi;
use wasmi::{errors::ModuleError, Error};

fn leb128_u32(mut value: u32) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            return out;
        }
    }
}

fn new_module(wasm: &[u8]) -> Result<wasmi::Module, Error> {
    let engine = wasmi::Engine::default();
    wasmi::Module::new(&engine, wasm)
}

fn assert_rejected_as_parser_err(result: Result<wasmi::Module, Error>, case: &str) {
    match result {
        Err(Error::Module(ModuleError::Parser(_))) => {}
        Err(other) => panic!("{case}: rejected by the wrong layer: {other}"),
        Ok(_) => panic!("{case}: accepted a module that ends before its sections do"),
    }
}

fn assert_rejected_as_malformed(result: Result<wasmi::Module, Error>, case: &str) {
    match result {
        Err(Error::Module(ModuleError::Malformed(_))) => {}
        Err(other) => panic!("{case}: rejected by the wrong layer: {other}"),
        Ok(_) => panic!("{case}: accepted a malformed module"),
    }
}

/// A type section declaring a ~4 GiB body, supplying one byte of it.
#[test]
fn declared_section_size_past_end_of_module_is_rejected() {
    let mut wasm = Vec::new();
    wasm.extend_from_slice(b"\x00asm");
    wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    wasm.push(0x01); // type section id
    wasm.extend_from_slice(&leb128_u32(u32::MAX)); // declared body length
    wasm.push(0x00); // the only byte of body actually present

    assert!(wasm.len() < 20, "the payload itself must stay tiny");
    assert_rejected_as_parser_err(new_module(&wasm), "type section");
}

/// The code section declares its body length the same way, and additionally
/// declares a length per function body. Both must run off the end safely.
#[test]
fn declared_code_section_size_past_end_of_module_is_rejected() {
    let prefix = {
        let mut wasm = Vec::new();
        wasm.extend_from_slice(b"\x00asm");
        wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        wasm.extend_from_slice(&[0x01, 0x04, 0x01, 0x60, 0x00, 0x00]); // type: one () -> ()
        wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]); // function: one func of type 0
        wasm
    };

    // The code section itself claims ~4 GiB.
    let mut section_overrun = prefix.clone();
    section_overrun.push(0x0a);
    section_overrun.extend_from_slice(&leb128_u32(u32::MAX));
    section_overrun.push(0x01); // one function body, per the declared count
    assert_rejected_as_parser_err(new_module(&section_overrun), "code section body");

    // The section length is honest, but the single function body claims ~4 GiB.
    let mut body_overrun = prefix;
    let mut body = vec![0x01]; // one function body
    body.extend_from_slice(&leb128_u32(u32::MAX)); // that body's declared size
    body_overrun.push(0x0a);
    body_overrun.extend_from_slice(&leb128_u32(body.len() as u32));
    body_overrun.extend_from_slice(&body);
    assert_rejected_as_parser_err(new_module(&body_overrun), "function body");
}

#[test]
fn declared_entry_count_is_bounded_by_section_size() {
    // (section id, declared count) at each section's `MAX_WASM_*` ceiling.
    let cases: &[(u8, u32, &str)] = &[
        (1, 1_000_000, "types"),
        (2, 1_000_000, "imports"),
        (3, 1_000_000, "functions"),
        (4, 1_000_000, "tables"),
        (5, 1_000_000, "memories"),
        (6, 1_000_000, "globals"),
        (7, 100_000, "exports"),
        (9, 100_000, "elements"),
        (11, 100_000, "data"),
    ];

    for (id, count, name) in cases {
        let mut wasm = Vec::new();
        wasm.extend_from_slice(b"\x00asm");
        wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        let body = leb128_u32(*count);
        wasm.push(*id);
        wasm.extend_from_slice(&leb128_u32(body.len() as u32));
        wasm.extend_from_slice(&body);

        assert_rejected_as_malformed(new_module(&wasm), name);
    }
}
