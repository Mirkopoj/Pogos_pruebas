use std::sync::mpsc::{Receiver, TryRecvError};
use crate::{to_bytes, TestData, TESTDATALEN};

pub trait ConvertTest {
    fn to_bytes(&self) -> [u8;TESTDATALEN];
}

impl ConvertTest for TestData {
    fn to_bytes(&self) -> [u8;TESTDATALEN] {
       to_bytes(*self) 
    }
}

pub fn ver_estado (rx: &Receiver<bool>) {
    match rx.try_recv() {
        Ok(msg) => {
            if msg { esperar(rx); }
        },
        Err(why) => {
            if why != TryRecvError::Empty { 
                panic!("Perdimos la pausa para las pruebas");
            }
        }
    }
}

fn esperar (rx: &Receiver<bool>) {
    while rx.recv().expect("Perdimos la pausa para las prueba, esperando") { }
}
