===============================================================================
==                        MP2ROMtracesdump for STM32MP2                      ==
===============================================================================

These binaries permits to display STM32MP2x bootrom traces on UART @115200 bauds, 8bits, 1 stop bit, no parity, no CTS/RTS.
According to used binary, the traces are output on the designated UART and GPIO pin.
For STM32MP23 or STM32MP25:
- MP2ROMtracesdump_USART2_PA4: traces output on USART2 using PA4 GPIO pin
- MP2ROMtracesdump_UART4_PH7: traces output on UART4 using PH7 GPIO pin
- MP2ROMtracesdump_UART5_PG9: traces output on UART5 using PG9 GPIO pin
- MP2ROMtracesdump_USART6_PF13: traces output on USART6 using PF13 GPIO pin
- MP2ROMtracesdump_UART7_PH4: traces output on UART7 using PH4 GPIO pin

For STM32MP21:
- MP21ROMtracesdump_USART2_PA4: traces output on USART2 using PA4 GPIO pin
- MP21ROMtracesdump_USART2_PC1: traces output on USART2 using PC1 GPIO pin
- MP21ROMtracesdump_UART4_PH7: traces output on UART4 using PH7 GPIO pin
- MP21ROMtracesdump_UART5_PG9: traces output on UART5 using PG9 GPIO pin
- MP21ROMtracesdump_USART6_PF13: traces output on USART6 using PF13 GPIO pin
- MP21ROMtracesdump_UART7_PH4: traces output on UART7 using PH4 GPIO pin

For each MP2xROMtracesdump, 2 binaries are available:
- xxx.stm32: binary with header, unsigned, to be run on a CLOSED_UNLOCKED chip
- xxx.bin: binary without header to be signed by customer to be run on CLOSED_LOCKED_PROVD chip

This binary has to be flashed on flash device (SD card, EMMC, Parallel NAND, SNOR, SNAND or HYPERFLASH) or it can be used for USB or UART boot.
On flash device, it can be flashed in FSBL1 position or in FSBL2 position as backup FSBL in case of FSBL1 failed loading.
This binary can only be used on CA35 Aarch64 TDCID, it cannot be executed on CM33.

When booting on this binary, the traces are directly output on the UART pin in plain text.
Here is an example of output traces for a boot on SD card on STM32MP25 platform:

===================================================
== STM32MP25 cut2.x bootrom display traces V1.00 ==
== Date of compilation : Jun 19 2024 - 14:52:24  ==
===================================================
Executed on Cortex A35 - Aarch64
Successfully booted on interface SD instance 1
Reset reason : POR reset
Chip state : CLOSED_UNLOCKED
==================================================================
Bootrom traces
==================================================================

[INFO] - BOOTCORE_FreezeIWDG12Clocks >
[INFO] - BOOTCORE_HwResetPOR >
[INFO] - BOOTCORE_VarLastValidAppRstMask ( 0x00000015 ) >
[INFO] - BOOTCORE_VarLastValidCpu1RstMask ( 0x00000003 ) >
[INFO] - BOOTCORE_VarLastValidPwrResetSource ( 0x00000000 ) >
[INFO] - BOOTCORE_ValRegHwRstSclrr ( 0x00000000 ) >
[INFO] - BOOTCORE_ValRegC1HwRstSclrr ( 0x00000000 ) >
[INFO] - BOOTCORE_ValRegC1BootRstSclrr ( 0x00002035 ) >
[INFO] - BOOTCORE_ChipModeClosedUnlocked >
[INFO] - BOOTCORE_LogicalResetSystem >
[INFO] - BOOTCORE_BootActionSecureBootProcess >
[INFO] - BOOTCORE_BootPinsSrcSel ( 0x00000001 ) >
[INFO] - BOOTCORE_BootCfgOtpWordValue ( 0x01000000, 0x00000000, 0x00000000 ) >
[INFO] - BOOTCORE_OtpBootpinLayoutSrc ( 0x00000001 ) >
[INFO] - BOOTCORE_OtpSrcSel ( 0x00000000 ) >
[INFO] - BOOTCORE_BootSrcIndexByBootPins ( 0x00000001 ) >
[INFO] - BOOTCORE_BootSrcIndex ( 0x00000001 ) >
[INFO] - BOOTCORE_SRCSEL_A35_SDCARD_SDMMC1_SDCARD_SDMMC1 >
[INFO] - BOOTCORE_OtpBootSrcDisableMaskVal ( 0x00000000 ) >
[INFO] - BOOTCORE_OtpBootUartInstanceDisableMaskVal ( 0x00000000 ) >
[INFO] - CSPID_CSPDEVICE ( 0x00000003 ) >
[INFO] - BOOTCORE_BootCfgAfmuxOtpWord1Value ( 0x00000000 ) >
[INFO] - BOOTCORE_BootCfgAfmuxOtpWord2Value ( 0x00000000 ) >
[INFO] - BOOTCORE_BootCfgAfmuxOtpWord3Value ( 0x00000000 ) >
[INFO] - BOOTCORE_BootCfgHseValue ( 0x00000000 ) >
[INFO] - BOOTCORE_EnabledSrcMaskVal ( 0x00000688 ) >
[INFO] - BOOTCORE_BootCfgHseValue ( 0x00000000 ) >
[INFO] - BOOTCORE_WDGStartAndUnfreeze >
[INFO] - BOOTCORE_UnFreezeIWDG12Clocks >
[INFO] - BOOTCORE_BootModeSECUREBOOT >
[INFO] - BOOTCORE_SecureBootLoadFsblA >
[INFO] - BOOTCORE_StartupWaitTime >
[INFO] - BOOTCORE_Pll14StartBegin >
[INFO] - BOOTCORE_NoCpuPllOtpBitValue ( 0x00000000 ) >
[INFO] - BOOTCORE_Pll1Locked >
[INFO] - BOOTCORE_CkMpuSwitchedOnPll1 >
[INFO] - BOOTCORE_Pll4Locked >
[INFO] - BOOTCORE_CkAxiSwitchedOnPll4 >
[INFO] - FLASHBOOT_HandleInit ( 0x00000003 ) >
[INFO] - SD_CardDetected >
[INFO] - SD_HighCapacityCardSDHCOrSDXC >
[INFO] - SD_TryFSBL1 >
[INFO] - SD_NotGPTFound >
[INFO] - SD_FsblsFound ( 0x00010000, 0x00050000 ) >
[INFO] - DWNLDMGR_FoundPaddingHeader >
[INFO] - SECBOOTCUSTOM_EnableDataCache >
[INFO] - SECBOOT_AuthenticationExtensionHeaderMissing >
[INFO] - SECBOOT_AuthImageLength ( 0x0000E400 ) >
[INFO] - SECBOOT_AuthImageEntryPoint ( 0x0E002600 ) >
[INFO] - SECBOOT_AuthDecisionIsJumpToImage >
[INFO] - SECBOOTCUSTOM_DisableDataCache >
[INFO] - BOOTCORE_AARCH64BootReq >
==================================================================
