/* First STM32U5A5 Flash page and two 4 KiB windows in SRAM1. */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 8K
  USER  : ORIGIN = 0x20001000, LENGTH = 4K
  RAM   : ORIGIN = 0x20002000, LENGTH = 4K
}

_mpu_flash_start = ORIGIN(FLASH);
_mpu_flash_end = ORIGIN(FLASH) + LENGTH(FLASH);
_mpu_supervisor_start = ORIGIN(RAM);
_mpu_supervisor_end = ORIGIN(RAM) + LENGTH(RAM);
_stack_start = _mpu_supervisor_end;
_stack_end = _stack_start - 2K;

SECTIONS
{
  .mpu_user (NOLOAD) :
  {
    _mpu_user_start = ABSOLUTE(.);
    KEEP(*(.mpu_user));
    _psp_bottom = ALIGN(., 8);
    . = ORIGIN(USER) + LENGTH(USER);
    _psp_top = .;
    _mpu_user_end = ABSOLUTE(.);
  } > USER
}
INSERT AFTER .uninit;

ASSERT(__euninit <= _stack_end, "Supervisor data overlaps MSP reserve");
ASSERT(_psp_bottom + 32 <= _psp_top, "No space for the basic PSP frame");
ASSERT(_mpu_user_start == ORIGIN(USER), "User window start shifted");
ASSERT(_mpu_user_end <= _mpu_supervisor_start, "SRAM windows overlap");
ASSERT(((_mpu_flash_start | _mpu_flash_end | _mpu_user_start |
         _mpu_user_end | _mpu_supervisor_start |
         _mpu_supervisor_end) & 31) == 0, "Unaligned MPU window");
