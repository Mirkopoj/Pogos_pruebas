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

#[derive(Copy, Clone, PartialEq, Default, Debug)]
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
    ret[14..22].copy_from_slice(&struct_in.tension2.to_be_bytes());
    ret[22..30].copy_from_slice(&struct_in.tension3.to_be_bytes());

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
    clear_background(BEIGE);
    
    let x = 95.0;
    let y = 100.0;
    let y_step = 30.0;
    let font_size = 40.0;

    draw_rectangle(x+8.0, y-10.0, 142.0, (y_step*11.0)+1.0, LIGHTGRAY);
    if test_data.programdo_test {
        for rect in 0..=test_data.abc {
            let offset = rect as f32;
            draw_rectangle(x+8.0, y+(y_step*2.1)+2.0+offset*y_step, 72.0, (y_step*1.0)+1.0, GREEN);
        }
        for rect in 0..test_data.z_cont {
            let offset = rect as f32;

            let a = (rect>>0 & 1) == 1; 
            let b = (rect>>1 & 1) == 1; 
            let c = (rect>>2 & 1) == 1; 

            let z_calculado = (a^b)&(b|c);
            let z = (test_data.z>>rect)&1 == 1;
            
            let resultado = z == z_calculado;
            let color = if resultado {GREEN} else {RED};
            draw_rectangle(175.0, y+(y_step*2.1)+2.0+offset*y_step, 35.0, (y_step*1.0)+1.0, color);
        }
        for rect in 0..test_data.y_cont {
            let offset = rect as f32;

            let a = (rect>>0 & 1) == 1; 
            let b = (rect>>1 & 1) == 1; 
            let c = (rect>>2 & 1) == 1; 

            let y_calculado = !((!(a|c))^(!(b&c)));
            let y_real = (test_data.y>>rect)&1 == 1;
            
            let resultado = y_real == y_calculado;
            let color = if resultado {GREEN} else {RED};
            draw_rectangle(210.0, y+(y_step*2.1)+2.0+offset*y_step, 35.0, (y_step*1.0)+1.0, color);
        }
    }

    let t1x = 1000.0 + (test_data.tension1*100.0) as f32;
    draw_triangle(vec2(t1x, y),
                  vec2(t1x-6.0, y-10.0),
                  vec2(t1x+6.0, y-10.0),
                  BLACK
    );

    draw_rectangle(999.0, y-1.0, 352.0, y_step*2.0+2.0, BLACK);
    draw_rectangle(1000.0, y, 350.0, y_step*2.0, RED);
    draw_rectangle(1160.0, y, 10.0, y_step*2.0, GREEN);

    let t2x = 1000.0 + (test_data.tension2*100.0) as f32;
    draw_triangle(vec2(t2x, y+y_step*4.0),
                  vec2(t2x-6.0, y+y_step*4.0-10.0),
                  vec2(t2x+6.0, y+y_step*4.0-10.0),
                  BLACK
    );

    draw_rectangle(999.0, y+y_step*4.0-1.0, 352.0, y_step*2.0+2.0, BLACK);
    draw_rectangle(1000.0, y+y_step*4.0, 350.0, y_step*2.0, RED);
    draw_rectangle(1000.0, y+y_step*4.0, 20.0, y_step*2.0, GREEN);

    let t3x = 1000.0 + (test_data.tension3*100.) as f32;
    draw_triangle(vec2(t3x, y+y_step*8.0),
                  vec2(t3x-6.0, y+y_step*8.0-10.0),
                  vec2(t3x+6.0, y+y_step*8.0-10.0),
                  BLACK
    );

    draw_rectangle(999.0, y+y_step*8.0-1.0, 352.0, y_step*2.0+2.0, BLACK);
    draw_rectangle(1000.0, y+y_step*8.0, 350.0, y_step*2.0, RED);
    draw_rectangle(1320.0, y+y_step*8.0, 30.0, y_step*2.0, GREEN);

    draw_text("Z=(A+B)(B+C)", x+200.0, y+y_step, font_size, BLACK);
    draw_text("o", x+357.0, y+y_step, font_size, BLACK);
    draw_text("Y=(A+B)+(BC)", x+200.0, y+y_step*2.7, font_size, BLACK);
    draw_text("o", x+323.0, y+y_step*2.7, font_size, BLACK);
    draw_text("   ___   __", x+200.0, y+y_step*1.7, font_size, BLACK);
    draw_text("  __________", x+200.0, y+y_step*1.5, font_size, BLACK);

    draw_text("Tensiones", x+700.0, y-y_step, font_size, BLACK);
    draw_text("Tension 1", x+700.0, y+y_step*1.0, font_size, BLACK);
    draw_text("Tension 2", x+700.0, y+y_step*5.0, font_size, BLACK);
    draw_text("Tension 3", x+700.0, y+y_step*9.5, font_size, BLACK);

    draw_text("+---+-+-+", x, y, font_size, BLACK);
    draw_text("|ABC|Z|Y|", x, y+y_step, font_size, BLACK);
    draw_text("|---+-+-|", x, y+y_step*2.0, font_size, BLACK);
    for i in 0u8..8 {
        let renglon = match (test_data.z_cont>test_data.abc, test_data.y_cont>test_data.abc) {
            (false, false) => {
                format!("|{:03b}|X|X|",i)
            },
            (true, false) => {
                format!("|{:03b}|{:1b}|X|",i,(test_data.z>>i)&1)
            },
            (true, true) => {
                format!(
                    "|{:03b}|{:1b}|{:1b}|",
                    i,
                    (test_data.z>>i)&1,
                    (test_data.y>>i)&1
                    )
            },
            _ => {"".to_string()},
        };
        draw_text(renglon.as_str(), x, y+(y_step*((i+3) as f32)), font_size, BLACK);
    }
    draw_text("+---+-+-+", x, y+y_step*11.0, font_size, BLACK);

}

const  NUMERODEPRUEBAS: usize = 1018;
pub fn prueba (pruebas_tx: &Sender<TestData>, pruebas_pausa_rx: &Receiver<bool>) -> Result<bool, SendError<TestData>> {
    println!("Entrando a las pruebas");
    let mut test_data: TestData = Default::default();
    let mut ret = true;

    println!("Pidiendo zy");
    let chipi = Chip::new("gpiochip3").expect("No se abri?? el chip, zy"); // open chip
    let ipts = Options::input([4,6]) 
        .consumer("my-inputs, zy"); 
    let inputs = chipi.request_lines(ipts).expect("Pedido de entradas rechazado, abc");

    println!("Pidiendo i2c");
    let mut i2c = I2c::from_path("/dev/i2c-0").expect("i2c");
    i2c.smbus_set_slave_address(0x08, false).expect("addr");
    i2c.smbus_write_byte(42).expect("Write inicial");//Sincronizar el pic
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
                abc_put(abc, &mut i2c, &mut test_data);
                print!("|{:03b}|",abc);
                sleep(Duration::from_millis(1));
                ret&=get_z(&mut test_data, &inputs, abc);
            },
            1..=16 if i%2 == 0 => {
                let abc = ((i/2)-1) as u8;
                ret&=get_y(&mut test_data, &inputs, abc);
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
    let z_medido = inputs.get_values([Some(false),None]).expect("No se ley?? z")[0].expect("No hab??a z");

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
    let y_medido = inputs.get_values([None,Some(false)]).expect("No se ley?? y")[1].expect("No hab??a y");

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
        .expect("Fall?? pickle id");

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
        .expect("Fall?? pickle blank");

    clear.stdin.take().expect("No se abri?? el stdin").write(b"y").expect("No se escribi??");
    let mut respuesta: [u8;50] = [0;50];
    clear.stdout.expect("No se abri?? el stdout").read(&mut respuesta).expect("Fall?? leer stdout");
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
        .expect("Fall?? pickle program");

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
        .expect("Fall?? pickle verify");

    let verstr = String::from_utf8(verify.stdout).expect("No se puede convertir verify");

    ret &= if verstr.contains("Fail: 0"){
        true
    } else {
        false
    };

    ret
}

fn abc_put(abc: u8, i2c: &mut I2c<File>, test_data: &mut TestData){
    i2c.smbus_write_byte(abc).expect("Write, abc");
    test_data.abc=abc;
}
