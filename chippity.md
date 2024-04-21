### Goals
- Familiarize myself with the core facets of Rust and get a feel for language semantics and software design in Rust.
- Have a program that may serve as a template to return to for test driving libraries and running quick turnaround experiments.

#### Project Design:

- Aside from the CHIP-8 spec—make as few assumptions about the system the interpreter will run on—keeping the CHIP-8 VM ( `Chip8` ) as compact as possible to be able to port to embedded environments. This means that frontend components should be fully decoupled and libraries used to back `Chip8` itself, if any, should be `no_std` capable.
- To some extent, convey an emulator's system model. This project will be tailored to CHIP-8, but one should be able to draw inpsiration from portions of this project in their own emulators, perhaps for a guest system with a different ISA!

---

### References & Credits
- *CHIP-8 Wikipedia Page*: https://en.wikipedia.org/wiki/CHIP-8
- *mattmikolay's CHIP-8 Technical Reference*: https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference
- *Cowgod's CHIP-8 Technical Reference*: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
- *Timendus' CHIP-8 Test Suite*: https://github.com/Timendus/chip8-test-suite

A collection of compatible ROMs may be found on [Zophar's Domain](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html) and John Earnest's [CHIP-8 community archive](https://johnearnest.github.io/chip8Archive/?sort=platform#chip8).
