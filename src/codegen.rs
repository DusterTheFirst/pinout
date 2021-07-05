use crate::{
    args::Language,
    netlist::{AssociatedComponent, Sheet},
};
use std::io::Write;

mod c;

type CodeGenerator = for<'t> fn(
    w: &mut dyn Write,
    sheet: &Sheet<'t>,
    component: &AssociatedComponent<'t>,
) -> anyhow::Result<()>;

pub fn get_generator(lang: Language) -> CodeGenerator {
    match lang {
        Language::C | Language::Cpp => c::generate,
        Language::Rust => unimplemented!("No Rust code generator is implemented"),
    }
}
