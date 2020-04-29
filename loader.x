INCLUDE memory-fe310.x
MEMORY
{
    FLASH : ORIGIN = 0x20000000, LENGTH = 4M
}


REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

/* Skip first 64k allocated for bootloader */
_stext = 0x20010000;

MEMORY 
{
    /* This is actually RAM, but we have relocatable code, so the origin does not matter */
    loader : ORIGIN = 0x0, LENGTH = 16k
}


SECTIONS {
    . = 0x0;

    PrgCode : {
        . = ALIGN(4);

        KEEP(*(PrgCode))
        KEEP(*(PrgCode.*))
        
        . = ALIGN(4);
    }  > loader

    PrgData . : {
        KEEP(*(PrgData))
        KEEP(*(PrgData.*))
    } > loader

    DeviceData . : {
        KEEP(*(DeviceData))
        KEEP(*(DeviceData.*))
    } > loader
}

INCLUDE link.x



