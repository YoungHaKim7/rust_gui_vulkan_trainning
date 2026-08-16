use rspirv::binary::Assemble;
use rspirv::binary::Disassemble;

fn main() {
    // Building
    let mut b = rspirv::dr::Builder::new();
    b.set_version(1, 0);
    b.capability(rspirv::Capability::Shader);
    b.memory_model(
        rspirv::AddressingModel::Logical,
        rspirv::MemoryModel::GLSL450,
    );
    let void = b.type_void();
    let voidf = b.type_function(void, vec![]);
    let fun = b
        .begin_function(
            void,
            None,
            rspirv::FunctionControl::DONT_INLINE | rspirv::FunctionControl::CONST,
            voidf,
        )
        .unwrap();
    b.begin_basic_block(None).unwrap();
    b.ret().unwrap();
    b.end_function().unwrap();
    b.entry_point(rspirv::ExecutionModel::Vertex, fun, "foo", vec![]);
    let module = b.module();

    // Assembling
    let code = module.assemble();
    assert!(code.len() > 20); // Module header contains 5 words
    assert_eq!(rspirv::MAGIC_NUMBER, code[0]);

    // Parsing
    let mut loader = rspirv::dr::Loader::new();
    rspirv::binary::parse_words(&code, &mut loader).unwrap();
    let module = loader.module();

    // Disassembling
    assert_eq!(
        module.disassemble(),
        "; SPIR-V\n\
         ; Version: 1.0\n\
         ; Generator: rspirv\n\
         ; Bound: 5\n\
         OpCapability Shader\n\
         OpMemoryModel Logical GLSL450\n\
         OpEntryPoint Vertex %3 \"foo\"\n\
         %1 = OpTypeVoid\n\
         %2 = OpTypeFunction %1\n\
         %3 = OpFunction  %1  DontInline|Const %2\n\
         %4 = OpLabel\n\
         OpReturn\n\
         OpFunctionEnd"
    );
}
