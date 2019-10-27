use crate::error::Result;
use log::info;
use std::collections::HashMap;
use mem::{PhysicalRead, VirtualRead};

pub mod error;
pub mod pe;
pub mod cache;
pub mod kernel;
pub mod win;

use win::{ProcessList, Windows};

/*
Options:
- supply cr3
- supply kernel hint
- supply pdb
- supply kernel offsets for basic structs (dumped windbg maybe)
*/

// TODO: impl Windows {}
pub fn init<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<Windows> {
    // TODO: add options to supply valid dtb

    // find dirtable base
    let stub_info = kernel::lowstub::find(mem)?;
    info!("arch={:?} va={:x} dtb={:x}", stub_info.arch, stub_info.va, stub_info.dtb);

    /*
        machine.cpu = Some(CPU{
            byte_order: ByteOrder::LittleEndian,
            arch: dtb.arch,
        })
    */

    // TODO: add option to supply a va hint
    // find ntoskrnl.exe base
    let kernel_base = kernel::ntos::find(mem, &stub_info)?;
    info!("kernel_base={:x}", kernel_base);

    // try to fetch pdb
    //let pdb = cache::fetch_pdb(pe)?;

    // system eprocess -> find
    let eprocess_base = kernel::sysproc::find(mem, &stub_info, kernel_base)?;
    info!("eprocess_base={:x}", eprocess_base);

    // grab pdb
    // TODO: new func or something in Windows impl
    let kernel_pdb = match cache::fetch_pdb_from_mem(mem, &stub_info, kernel_base) {
        Ok(p) => Some(p),
        Err(e) => {
            info!("unable to fetch pdb from memory: {:?}", e);
            None
        }
    };

    println!("kernel_pdb: {:?}", kernel_pdb.clone().unwrap());

    let mut win = Windows {
        kernel_stub_info: stub_info,
        kernel_base: kernel_base,
        eprocess_base: eprocess_base,
        kernel_pdb: kernel_pdb,
        kernel_structs: HashMap::new(),
    };

    // TODO: create fallback thingie which implements hardcoded offsets
    // TODO: create fallback which parses C struct from conf file + manual pdb
    // TODO: add class wrapper to Windows struct
    //let pdb = ; // TODO: add manual pdb option
    //let class = types::Struct::from(pdb, "_EPROCESS").unwrap();
    println!(
        "_EPROCESS::UniqueProcessId: {:?}",
        win.get_kernel_struct("_EPROCESS")
            .unwrap()
            .get_field("UniqueProcessId")
    );

    // PsLoadedModuleList / KDBG -> find

    // pdb, winreg?

    //pe::test_read_pe(mem, dtb, ntos)?;

    // TODO: copy architecture and

    let list = win.process_list();
    Ok(win)
}
