use std::sync::mpsc::{Receiver, Sender, SendError};

use macroquad::prelude::*;

pub mod funciones_internas;
use funciones_internas::*;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct TestData {
    /* Atributos que sean necesarios para comunicar los resultados de las pruebas */
}

/* Editar el valor de la constante de acuerdo al tamaño del array de bytes */
pub const  TESTDATALEN: usize = 0;
fn to_bytes(_struct_in: TestData) -> [u8;TESTDATALEN]{
    /* Funcion que permita condensar los datos en un array de bytes */
    [0;TESTDATALEN]
}

pub fn from_bytes(_bytes_in: &[u8]) -> TestData {
    /* Funcion inversa a to_bytes */
    Default::default()
}

pub fn resultados (_test_data: TestData) {
    clear_background(GREEN);
    /* Aqui va un hmi custom para las pruebas, utilizando macroquad */
    /* El espacio de pantalla disponible va de (20.0, 20.0) a (screen_width()-230.0,screen_height()-430.0) */
}

/* Editar el valor de la constante de acuerdo al numero de pruebas */
const  NUMERODEPRUEBAS: usize = 3;
pub fn prueba (pruebas_tx: &Sender<TestData>, pruebas_pausa_rx: &Receiver<bool>) -> Result<bool, SendError<TestData>> {
    let mut test_data: TestData = Default::default();
    for i in 0..NUMERODEPRUEBAS {
        ver_estado(pruebas_pausa_rx);
        match i {
            /* Dentro de este bloque llamar a las pruebas que se desen en orden */
            0 => {
                test0(&mut test_data);
            },
            1 => {
                test1();
            },
            2 => {
                test2();
            },
            _ => { },
        };

        pruebas_tx.send(test_data)?
    }

    Ok(true) //true para placas funcionales, false para placas defectuosas
}

/* Pruebas custom */
fn test0(_test_data: &mut TestData) {}
fn test1() {}
fn test2() {}
