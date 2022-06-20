# v4.2

# absolute path of source code root folder
path_source_tf_a     = ""
path_source_op_tee   = ""
path_source_u_boot   = ""
path_source_linux    = ""

# absolute path of elf files
path_elf_tf_a_bl2    = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/arm-trusted-firmware/debug/tf-a-bl2-sdcard.elf"
path_elf_tf_a_bl32   = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/arm-trusted-firmware/bl32/debug/tf-a-bl32-stm32mp15.elf"
path_elf_op_tee_bl32 = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/optee/debug/tee-stm32mp157c-dk2.elf"
path_elf_u_boot      = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/u-boot/debug/u-boot-stm32mp157c-dk2-trusted.elf"
path_elf_vmlinux     = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/kernel/vmlinux"

# absolute path of fip file
path_fip             = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/fip/fip-stm32mp157c-dk2-trusted.bin"

# override commands if not in PATH
# dtc and fiptool are provided by SDK for OpenSTLinux
# readelf and sed are required by https://wiki.st.com/stm32mpu/wiki/PC_prerequisites
DTC     = "dtc"
FIPTOOL = "fiptool"
READELF = "readelf"
SED     = "sed"

####################################################################
# Python code, do not modify

import os
import subprocess

bl32_load_addr = None

class STM32_gdb (gdb.Command):
    """grab info for gdb attach"""

    user_vars = ["path_source_tf_a", "path_source_op_tee", "path_source_u_boot", "path_source_linux",
                   "path_elf_tf_a_bl2", "path_elf_tf_a_bl32", "path_elf_op_tee_bl32", "path_elf_u_boot",
                   "path_elf_vmlinux", "path_fip", "DTC", "FIPTOOL", "READELF", "SED"]

    def __check_user_vars_defined(self):
        # Validate variables from user
        var_ndef = False
        for i in self.user_vars:
            if not i in globals():
                print("Variable not defined \"" + i + "\"")
                var_ndef = True
        if var_ndef:
            raise

    def __check_ext_tool(self, _name, _path, arg):
        try:
            subprocess.check_output([_path, arg])
        except:
            print("ERROR: cannot run \"" + _name + "\" program \"" + _path + "\"")
            raise

    # parse FIP to extract DT in FW_CONFIG and get load address of BL32
    def __get_bl32_offset_from_fip(self, _file):
        global bl32_load_addr

        if (not os.path.isfile(_file)):
            print("ERROR: FIP file \"" + _file + "\" not found")
            raise
        try:
            subprocess.check_output([FIPTOOL, "info", _file], stderr=subprocess.STDOUT)
        except:
            print("ERROR: file \"" + _file + "\" is not a FIP file")
            raise
        try:
            out = subprocess.check_output([FIPTOOL, "unpack", "--force", "--tos-fw", "/dev/null", _file], stderr=subprocess.STDOUT)
            if len(out):
                raise
        except:
            print("ERROR: FIP file \"" + _file + "\" don't have BL32 payload")
            raise
        try:
            out = subprocess.check_output(["sh", "-c",
                    FIPTOOL + " unpack --force --fw-config /dev/stdout " + _file + " | " \
                    + DTC + " -I dtb -O dts | " \
                    + SED + " -n '/{$/{h};/load-address/{G;/tos_fw /{s/>.*//;s/.*0x/0x/;p}}'"]).decode('utf-8')
            # print("result is " + out)
            bl32_load_addr = out
            return int(out, 16)
        except:
            print("ERROR: cannot extract BL32 offset from FIP file \"" + _file + "\"")
            raise

    # Check that _file is an ELF containing _sym and return the value of _sym
    def __check_elf_file(self, _descr, _file, _sym):
        if not os.path.isfile(_file):
            print("ERROR: file not found \"" + _file + "\" for " +_descr)
            raise
        try:
            f = open(_file, "rb")
            magic = f.read(4)
            f.close()
        except:
            print("ERROR: reading file  \"" + _file + "\"")
            raise

        if magic != b'\x7fELF':
            print("ERROR: not an elf file  \"" + _file + "\"")
            raise
        try:
            s = subprocess.check_output(["sh", "-c", READELF + " --syms " + _file + " | grep -w " + _sym]).decode('utf-8')
        except:
            print("ERROR: elf file \"" + _file + "\" does not provide the symbol " + _sym)
            raise
        return int(s.split(None, 2)[1], 16)

    # verify that _path contains _file
    def __check_source_dir(self, _prj_name, _path, _file):
        if not os.path.isfile(os.path.join(_path, _file)):
            print("WARNING: no " + _prj_name + " source code detected in folder \"" + _path + "\"")

    def __get_source_dir(self, _prj_name, _elf_file, _source_file):
        try:
            s = subprocess.check_output(["sh", "-c", READELF + " --string-dump=.debug_str " + _elf_file + " | grep " + _source_file + "$"]).decode('utf-8')
        except:
            print("Warning: cannot get path of " + _source_file + " from elf file \"" + _elf_file)
            return None
        return s.split(None)[-1].split(_source_file)[0]

    def __init__(self):
        print()
        use_optee = (gdb.convenience_variable("debug_trusted_bootchain") == 1)
        use_reset = (gdb.convenience_variable("debug_mode") == 0)
        debug_phase = gdb.convenience_variable("debug_phase")
        # TODO: print a summary of attach mode

        try:
            self.__check_user_vars_defined()

            self.__check_ext_tool("readelf", READELF, "--version")
            if debug_phase != 1 and not use_optee:
                self.__check_ext_tool("dtc",     DTC,     "-v")
                self.__check_ext_tool("fiptool", FIPTOOL, "help")
                self.__check_ext_tool("sed",     SED,     "--version")
                self.__get_bl32_offset_from_fip(path_fip)

            if use_reset or debug_phase == 1:
                self.__check_elf_file("TF-A BL2",  path_elf_tf_a_bl2,    "bl2_entrypoint")
            if debug_phase != 1 and not use_optee:
                self.__check_elf_file("TF-A BL32", path_elf_tf_a_bl32,   "sp_min_entrypoint")
            if debug_phase != 1 and use_optee:
                self.__check_elf_file("OP-TEE",    path_elf_op_tee_bl32, "tee_svc_do_call")
            if debug_phase == 3:
                self.__check_elf_file("U-Boot",    path_elf_u_boot,      "bootdelay_process")
            if debug_phase == 4:
                self.__check_elf_file("Linux",     path_elf_vmlinux,     "linux_banner")

            self.__check_source_dir("TF-A",   path_source_tf_a,   "common/bl_common.c")
            gdb.execute("directory " + path_source_tf_a)
            if debug_phase != 1 and use_optee:
                self.__check_source_dir("OP-TEE", path_source_op_tee, "core/tee/tee_svc.c")
                gdb.execute("directory " + path_source_op_tee)
            if debug_phase == 3:
                self.__check_source_dir("U-Boot", path_source_u_boot, "cmd/version.c")
                re_path = self.__get_source_dir("U-Boot", path_elf_u_boot, "cmd/version.c")
                if re_path:
                    gdb.execute("set substitute-path " + re_path + " " + path_source_u_boot)
            if debug_phase == 4:
                self.__check_source_dir("Linux",  path_source_linux,  "kernel/printk/printk.c")
                re_path = self.__get_source_dir("Linux", path_elf_vmlinux, "kernel/printk/printk.c")
                if re_path:
                    gdb.execute("set substitute-path " + re_path + " " + path_source_linux)

        except:
            gdb.execute("set confirm off")
            gdb.execute("quit")
        print()

STM32_gdb()
