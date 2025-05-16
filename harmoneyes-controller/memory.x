MEMORY {
    /*
        Note: The units of `K` are Kibibytes (KiB) where 1 KiB = 1024 bytes

        These values should correspond to the nRF52840 which is listed as
        having 1024 kB (1000 KiB) of flash memory and 256 kB (250 KiB) of RAM.

        The values are then offset by the flash and RAM requirements of the
        S140 Softdevice v7.3.0 which is listed as requiring 156 kB (just
        under 153 KiB) of flash memory and at a minimum 5.6 kB (specifically
        0x1678, 5752 bytes, or just under 6 KiB) of RAM.
    */
    FLASH : ORIGIN = 0x00000000 + 156K, LENGTH = 1024K - 156K - 4K
    STORAGE : ORIGIN = 0x00000000 + 1024K - 4K, LENGTH = 4K
    RAM : ORIGIN = 0x20000000 + 48K, LENGTH = 256K - 48K
}

__storage = ORIGIN(STORAGE);