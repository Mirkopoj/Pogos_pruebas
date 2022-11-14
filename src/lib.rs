use macroquad::prelude::*;

pub const  TESTDATALEN: usize = 0;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct TestData {
}

fn to_bytes(_struct_in: TestData) -> [u8;TESTDATALEN]{
    [0;TESTDATALEN]
}

pub trait ConvertTest {
    fn to_bytes(&self) -> [u8;TESTDATALEN];
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
