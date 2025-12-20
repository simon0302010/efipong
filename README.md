# Efipong

Efipong is a pong game running entirely in UEFI. This means that it doesn't even require an operating system to run. It can be installed onto any USB drive and booted directly from there.

## Installation

You can download a pre-built binary from the [releases page](https://github.com/simon0302010/efipong/releases).
> ⚠️ These binaries can be outdated. Please build from source for the latest version.

Clone the repository:

```bash
git clone https://github.com/simon0302010/efipong.git
cd efipong
```

### Building

Now build the project for your UEFI target:

For x86_64 target, use:

```bash
cargo build --target x86_64-unknown-uefi --release
```

For aarch64 target, use:

```bash
cargo build --target aarch64-unknown-uefi --release
```

For 32-bit x86 target, use:

```bash
cargo build --target i686-unknown-uefi --release
```

You should now have the binary located at `target/<target>/release/efipong.efi`.

### Preparing a Bootable USB Drive

1. Format a USB drive to FAT32.
2. Create a directory structure on the USB drive as follows: `EFI/BOOT/`.
3. Copy the built `efipong.efi` file to the `EFI/BOOT/` directory and rename it to `BOOTX64.EFI` for x86_64, `BOOTAA64.EFI` for aarch64, or `BOOTIA32.EFI` for i686.

## Running

1. Insert the USB drive into the target machine.
2. Boot the machine and enter the UEFI boot menu (usually by pressing a key like F12, F10 or F8 during startup).
3. Select the USB drive to boot from it.
4. The game should start automatically.

### Controls

- Move left paddle: `W` (up), `S` (down)
- Move right paddle: `Up Arrow` (up), `Down Arrow` (down)
- Quit game: `Q`
- Start/Restart game: `Space`

### Rules

- First player to reach 11 points wins.
- A point is scored when the ball passes the players's paddle.