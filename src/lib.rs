use std::sync::mpsc::{Receiver, Sender, SendError};

use macroquad::prelude::*;

pub mod funciones_internas;
use funciones_internas::*;

use std::process::{Command, Stdio};
use std::io::{Write, Read};
use std::str;
use std::time::Duration;
use std::thread::sleep;
use gpiod::{Chip, Options, Lines, Input};
extern crate i2c_linux;
use i2c_linux::I2c;
use std::fs::File;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct TestData {
    programdo_test: bool,
    verificado_test: bool,
    abc: u8,
    z: u8,
    z_cont: u8,
    y: u8,
    y_cont: u8,
    tension1: f64,
    tension2: f64,
    tension3: f64,
    programdo_bueno: bool,
    verificado_bueno: bool,
}

pub const  TESTDATALEN: usize = 30;
fn to_bytes(struct_in: TestData) -> [u8;TESTDATALEN]{
    let mut ret = [0;TESTDATALEN];

    if struct_in.programdo_test  { ret[0] |= 1; }
    if struct_in.verificado_test { ret[0] |= 2; }
    if struct_in.programdo_bueno { ret[0] |= 3; }
    if struct_in.verificado_bueno{ ret[0] |= 4; }

    ret[1] = struct_in.abc;
    ret[2] = struct_in.z;
    ret[3] = struct_in.z_cont;
    ret[4] = struct_in.y;
    ret[5] = struct_in.y_cont;

    ret[06..14].copy_from_slice(&struct_in.tension1.to_be_bytes());
    ret[14..22].copy_from_slice(&struct_in.tension1.to_be_bytes());
    ret[22..30].copy_from_slice(&struct_in.tension1.to_be_bytes());

    ret
}

pub fn from_bytes(bytes_in: &[u8]) -> TestData {
    TestData { 
        programdo_test: bytes_in[0]&1 == 1, 
        verificado_test: bytes_in[0]&2 == 2, 
        abc:    bytes_in[1], 
        z:      bytes_in[2],
        z_cont: bytes_in[3],
        y:      bytes_in[4],
        y_cont: bytes_in[5],
        tension1: f64::from_be_bytes(bytes_in[06..14].try_into().expect("Slice tension1 de largo incorrecto")),
        tension2: f64::from_be_bytes(bytes_in[14..22].try_into().expect("Slice tension1 de largo incorrecto")),
        tension3: f64::from_be_bytes(bytes_in[22..30].try_into().expect("Slice tension1 de largo incorrecto")),
        programdo_bueno: bytes_in[0]&3 == 3, 
        verificado_bueno: bytes_in[0]&4 == 4, 
    }
}

pub fn resultados (test_data: TestData) {
    clear_background(GREEN);
    
    draw_text("+---+-----+-----+-----+-----+-----+-----+\n|ABC|Zcalc|Zreal|  == |Ycalc|Yreal|  == |\n|---+-----+-----+-----+-----+-----+-----|", 95.0, 100.0, 60.0, BLACK);

}

const  NUMERODEPRUEBAS: usize = 1018;
pub fn prueba (pruebas_tx: &Sender<TestData>, pruebas_pausa_rx: &Receiver<bool>) -> Result<bool, SendError<TestData>> {
    println!("Entrando a las pruebas");
    let mut test_data: TestData = Default::default();
    let mut ret = true;

    println!("Pidiendo zy");
    let chipi = Chip::new("gpiochip3").expect("No se abrió el chip, zy"); // open chip
    let ipts = Options::input([4,6]) 
        .consumer("my-inputs, zy"); 
    let inputs = chipi.request_lines(ipts).expect("Pedido de entradas rechazado, abc");

    println!("Pidiendo i2c");
    let mut i2c = I2c::from_path("/dev/i2c-0").expect("i2c");
    i2c.smbus_set_slave_address(0x08, false).expect("addr");
    i2c.smbus_write_byte(42).expect("Write");//Sincronizar el pic
    let mut tensiones: [f64;3] = [0.0;3];

    for i in 0..=NUMERODEPRUEBAS {
        ver_estado(pruebas_pausa_rx);
        match i {
            0 => {
                ret&=programar_y_verificar_test(&mut test_data);
                println!("+--------------+");
                println!("|Programar Test|");
                println!("+--------------+");
                println!("+---+-----+-----+-----+-----+-----+-----+");
                println!("|ABC|Zcalc|Zreal|  == |Ycalc|Yreal|  == |");
                println!("|---+-----+-----+-----+-----+-----+-----|");
            },
            1..=16 if i%2 == 1 => {
                let abc = (i/2) as u8;
                abc_put(abc, &mut i2c);
                print!("|{:03b}|",abc);
                sleep(Duration::from_millis(1));
                ret&=get_z(&mut test_data, &inputs, abc);
                //if !ret {println!("Falló ^^^^");}
            },
            1..=16 if i%2 == 0 => {
                let abc = ((i/2)-1) as u8;
                ret&=get_y(&mut test_data, &inputs, abc);
                //if !ret {println!("Falló ^^^^");}
            },
            17..=1017 => {
                adc(&mut test_data, i-17, &mut tensiones, &mut i2c);
            },
            1018 => {
                ret&=programar_y_verificar_bueno(&mut test_data);
                println!("+---+-----+-----+-----+-----+-----+-----+");
                println!("+---------------+");
                println!("|Programar Bueno|");
                println!("+---------------+");
            },
            _ => { },
        };

        pruebas_tx.send(test_data)?
    }

    ret &= (test_data.tension1<1.7) & (test_data.tension1>1.6);
    ret &= test_data.tension2<0.2;
    ret &= test_data.tension3>3.2;
    println!("+------------------------------+");
    println!("|Tension1: {:.18}|", test_data.tension1);
    println!("|Tension2: {:.18}|", test_data.tension2);
    println!("|Tension3: {:.18}|", test_data.tension3);
    println!("+------------------------------+");
    println!("+---------------+");
    println!("|Aprobado: {:5}|", ret);
    println!("+---------------+");

    Ok(ret) //true para placas funcionales, false para placas defectuosas
}

/* Pruebas custom */
fn programar_y_verificar_test(struct_in: &mut TestData) -> bool {
    struct_in.verificado_test = programar_y_verificar("/home/dietpi/TestCode.hex"); 
    struct_in.programdo_test = true;
    struct_in.verificado_test
}

fn programar_y_verificar_bueno(struct_in: &mut TestData) -> bool {
    struct_in.verificado_bueno = programar_y_verificar("/home/dietpi/FinalCode.hex"); 
    struct_in.programdo_bueno = true;
    struct_in.verificado_bueno
}

fn get_z(struct_in: &mut TestData, inputs: &Lines<Input>, conv: u8) -> bool {
    let z_medido = inputs.get_values([Some(false),None]).expect("No se leyó z")[0].expect("No había z");

    let a = (conv>>0 & 1) == 1; 
    let b = (conv>>1 & 1) == 1; 
    let c = (conv>>2 & 1) == 1; 

    let z_calculado = (a^b)&(b|c);
    
    let z = if z_medido { 1 } else { 0 };
    struct_in.z |= z<<struct_in.z_cont;
    struct_in.z_cont += 1;

    let ret = z_calculado == z_medido;
    print!("{:5}|{:5}|{:5}|",z_calculado,z_medido,ret);

    ret
}

fn get_y(struct_in: &mut TestData, inputs: &Lines<Input>, conv: u8) -> bool {
    let y_medido = inputs.get_values([None,Some(false)]).expect("No se leyó y")[1].expect("No había y");

    let a = (conv>>0 & 1) == 1; 
    let b = (conv>>1 & 1) == 1; 
    let c = (conv>>2 & 1) == 1; 

    let y_calculado = !((!(a|c))^(!(b&c)));
    
    let y = if y_medido { 1 } else { 0 };
    struct_in.y |= y<<struct_in.y_cont;
    struct_in.y_cont += 1;

    let ret = y_calculado == y_medido;
    println!("{:5}|{:5}|{:5}|",y_calculado,y_medido,ret);

    ret
}

fn adc(struct_in: &mut TestData,
       cont: usize,
       tension_tot: &mut [f64;3],
       i2c: &mut I2c<File>,
       ){
    i2c.smbus_write_byte(42).expect("Write, adc");//Sincronizar el pic
                                             
    for i in 0..3 {
        let adch = i2c.smbus_read_byte().expect("ReadH") as u16;
        let adcl = i2c.smbus_read_byte().expect("ReadL") as u16;
        let adc:u16 = (adch <<8) + adcl;
        let tension = adc as f64 * 0.003225806452;
        tension_tot[i] += tension;
    }

    struct_in.tension1 = tension_tot[0]/cont as f64;
    struct_in.tension2 = tension_tot[1]/cont as f64;
    struct_in.tension3 = tension_tot[2]/cont as f64;
}

/* Auxiliares de pruebas */
fn programar_y_verificar(path_to_hex: &'static str) -> bool {
    let mut ret = true;

    let id = Command::new("p16")
        .arg("id")
        .output()
        .expect("Falló pickle id");

    let idstr = String::from_utf8(id.stdout).expect("No se puede convertir id");

    ret &= if idstr.contains("pic16_read_config_memory"){
        false
    } else {
        true
    };

    let mut clear = Command::new("p16")
        .arg("blank")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Falló pickle blank");

    clear.stdin.take().expect("No se abrió el stdin").write(b"y").expect("No se escribió");
    let mut respuesta: [u8;50] = [0;50];
    clear.stdout.expect("No se abrió el stdout").read(&mut respuesta).expect("Falló leer stdout");
    let clrstr = str::from_utf8(&respuesta).expect("No se puede convertir program");

    ret &= if clrstr.contains("pic16_read_config_memory"){
        false
    } else {
        true
    };

    let program = Command::new("p16")
        .arg("program")
        .arg(path_to_hex)
        .output()
        .expect("Falló pickle program");

    let prgstr = String::from_utf8(program.stdout).expect("No se puede convertir program");

    ret &= if prgstr.contains("pic16_read_config_memory"){
        false
    } else {
        true
    };

    let verify = Command::new("p16")
        .arg("verify")
        .arg(path_to_hex)
        .output()
        .expect("Falló pickle verify");

    let verstr = String::from_utf8(verify.stdout).expect("No se puede convertir verify");

    ret &= if verstr.contains("Fail: 0"){
        true
    } else {
        false
    };

    ret
}

fn abc_put(abc: u8, i2c: &mut I2c<File>){
    i2c.smbus_write_byte(abc).expect("Write, abc");
}
