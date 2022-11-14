use macroquad::prelude::*;
use modulos_comunes::ConvertTest;

pub const  TESTDATALEN: usize = 0;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct TestData {
}

fn to_bytes(struct_in: TestData) -> [u8;TESTDATALEN]{
}

impl ConvertTest for TestData {
    fn to_bytes(&self) -> [u8;TESTDATALEN] {
       to_bytes(*self) 
    }
}

pub fn prueba () {
}

pub fn resultados () {
    clear_background(GREEN);
}
