use rspirv::binary::Assemble;
use rspirv::binary::Disassemble;
use rspirv::spirv::{AddressingModel, Capability, ExecutionModel, FunctionControl, MemoryModel};

fn main() {
    // Building
    let mut b = rspirv::dr::Builder::new();
    b.set_version(1, 0);
    b.capability(Capability::Shader);
    b.memory_model(AddressingModel::Logical, MemoryModel::GLSL450);
    let void = b.type_void();
    let voidf = b.type_function(void, vec![]);
    let fun = b
        .begin_function(void, None, FunctionControl::DONT_INLINE | FunctionControl::CONST, voidf)
        .unwrap();
    b.begin_block(None).unwrap();
    b.ret().unwrap();
    b.end_function().unwrap();
    b.entry_point(ExecutionModel::Vertex, fun, "foo", vec![]);
    let module = b.module();

    // Assembling
    let code = module.assemble();
    assert!(code.len() > 20); // Module header contains 5 words
    assert_eq!(rspirv::spirv::MAGIC_NUMBER, code[0]);

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
