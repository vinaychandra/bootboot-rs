# BOOTBOOT-RS

A rust wrapper for the [BOOTBOOT](https://gitlab.com/bztsrc/bootboot) bootloader.

Add this crate as a dependency to your bootboot kernel and use methods on the `BootBoot` struct
to retrieve details from the bootloader.

### Sample

See [this](https://gitlab.com/bztsrc/bootboot/-/tree/master/mykernel/rust) for a sample

### Quick steps

- Create a rust project - `cargo new sample`
- Edit `main.rs`

  ```
  #![no_std]
  #![no_main]

  use core::panic::PanicInfo;

  #[no_mangle] // don't mangle the name of this function. Acts as the entry point.
  fn _start() -> ! {
      loop {}
  }

  #[panic_handler]
  fn panic(_info: &PanicInfo) -> ! {
      loop {}
  }
  ```

- Create a triplet for your target architecture.

  - Example `x86_64`
    ```
    {
        "arch": "x86_64",
        "code-model": "kernel",
        "data-layout": "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
        "disable-redzone": true,
        "dynamic-linking": false,
        "eliminate-frame-pointer": false,
        "exe-suffix": "",
        "executables": true,
        "features": "-mmx,-sse,+soft-float",
        "has-elf-tls": true,
        "has-rpath": false,
        "linker-flavor": "ld.lld",
        "linker": "rust-lld",
        "llvm-target": "x86_64-unknown-none",
        "no-compiler-rt": true,
        "no-default-libraries": true,
        "os": "none",
        "panic-strategy": "abort",
        "position-independent-executables": false,
        "relocation-model": "static",
        "target-c-int-width": "32",
        "target-endian": "little",
        "target-pointer-width": "64",
        "pre-link-args": {
            "ld.lld": ["--script=./link.ld"]
        }
    }
    ```
  - Example `aarch64`

    ```
    {
        "unsupported-abis": [
            "stdcall",
            "fastcall",
            "vectorcall",
            "thiscall",
            "win64",
            "sysv64"
        ],
        "arch": "aarch64",
        "data-layout": "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
        "disable-redzone": true,
        "dynamic-linking": false,
        "eliminate-frame-pointer": false,
        "exe-suffix": "",
        "executables": true,
        "features": "+strict-align,-neon,-fp-armv8",
        "has-elf-tls": true,
        "has-rpath": false,
        "linker-flavor": "ld.lld",
        "linker": "rust-lld",
        "llvm-target": "aarch64-unknown-none",
        "max-atomic-width": 128,
        "no-compiler-rt": true,
        "no-default-libraries": true,
        "os": "none",
        "panic-strategy": "abort",
        "position-independent-executables": false,
        "relocation-model": "pic",
        "target-c-int-width": "32",
        "target-cpu": "cortex-a53",
        "target-endian": "little",
        "target-pointer-width": "64",
        "pre-link-args": {
            "ld.lld": ["--script=./link.ld"]
        }
    }
    ```

- Add a linker file `link.ld`. Customize this as needed.

  - Example

    ```
    KERNEL_OFFSET = 0xfffffffff0000000;

    PHDRS
    {
        boot PT_LOAD FILEHDR PHDRS;
        tls PT_TLS;
    }
    SECTIONS
    {
        . = KERNEL_OFFSET;
        mmio    = .; . += 0x4000000;
        fb      = .; . += 0x3E00000;
        bootboot    = .; . += 4096;
        environment = .; . += 4096;

        .text . + SIZEOF_HEADERS : AT(ADDR(.text) - . + SIZEOF_HEADERS) {
            KEEP(*(.text.boot)) *(.text .text.* .gnu.linkonce.t*)   /* code */
            . = ALIGN(4096);
        } :boot

        .rodata : {
            *(.rodata*)
            . = ALIGN(4096);
        } :boot

        .data : {
            *(.data*)
            . = ALIGN(4096);
        } :boot

        .bss : {
            *(.bss*)
            . = ALIGN(4096);
        } :boot

        .got : {
            *(.got*)
            . = ALIGN(4096);
        } :boot

        .tdata : {
            __tdata_start = .;
            *(.tdata*)
            . = ALIGN(4096);
            __tdata_end = .;
        } :boot :tls


        .tbss : {
            __tbss_start = .;
            *(.tbss*)
        } :boot :tls

        /*TBSS has no size. So, we force it to have size here.*/
        __tbss_align = ALIGNOF(.tbss);
        . += SIZEOF(.tbss);
        __tbss_end = .;


        /DISCARD/ : { *(.eh_frame) *(.comment) }
    }
    ```

- Build the code
  - Example: `cargo xbuild --target ./triplets/aarch64.json`
- This above generates the target elf file for the bootloader to load from.
- Run the following commands to generate an initrd structure
  ```
  mkdir initrd
  mkdir initrs/sys
  cp <output elf> initrd/sys/core
  ```
- Create a `config` file:
  ```
  // BOOTBOOT loader configuration
  screen=800x600
  kernel=sys/core
  ```
- Create a `mkbootimg.json` file:
  ```
  {
    "diskguid": "00000000-0000-0000-0000-000000000000",
    "config": "./config",
    "initrd": {
        "type": "tar",
        "gzip": false,
        "directory": "./initrd"
    },
    "partitions": [
        {
            "type": "fat32",
            "size": 130
        }
    ]
  }
  ```
- Download binaries from the [bootboot](https://gitlab.com/bztsrc/bootboot) repo.
- Run `./mkbootimg mkbootimg.json kernel.img` to generate a boot image.
- To run under qemu, run the following,
  - aarch64 - `bootboot.img` is available in the above repo.
    ```
    qemu-system-aarch64 -M raspi3 -kernel bootboot.img -drive file=kernel.img,if=sd,format=raw -serial stdio
    ```
  - x86_64 EFI - Need OVMF (Example: `/usr/share/qemu/OVMF.fd`)
    ```
    qemu-system-x86_64 -bios <OVMF PATH> -m 128 -drive file=kernel.img,format=raw -serial stdio -no-shutdown -no-reboot
    ```

### How to update the repo

#### Update bootboot bindings

- Copy bootboot.h from bootboot repo
- Add `#include <stdint.h>` in the defines
- Run `bindgen bootboot/bootboot.h -o src/bootboot_bindings.rs --use-core --ctypes-prefix=cty`
