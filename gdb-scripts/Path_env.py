# v5.0

# absolute path of source code root folder
path_source_tf_a     = ""
path_source_op_tee   = ""
path_source_u_boot   = ""
path_source_linux    = ""

# absolute path of elf files
path_elf_tf_a_bl2    = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/arm-trusted-firmware/debug/tf-a-bl2-stm32mp15-optee-sdcard.elf"
path_elf_op_tee_bl32 = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/optee/debug/tee-stm32mp157f-dk2.elf"
path_elf_u_boot      = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/u-boot/debug/u-boot-stm32mp15-default.elf"
path_elf_vmlinux     = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/kernel/vmlinux"

# absolute path of fip file
path_fip             = "/opt/openstlinux/build-openstlinuxweston-stm32mp1/tmp-glibc/deploy/images/stm32mp1/fip/fip-stm32mp157f-dk2-optee-sdcard.bin"

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

# relocable elf files with dummy load address. Get them from FIP
load_add_optee_bl32 = None

class STM32_gdb (gdb.Command):
    """grab info for gdb attach"""

    user_vars = ["path_source_tf_a", "path_source_op_tee", "path_source_u_boot", "path_source_linux",
                   "path_elf_tf_a_bl2", "path_elf_op_tee_bl32", "path_elf_u_boot",
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

    # parse FIP to extract DT in FW_CONFIG and get load address
    def __get_load_addr_from_fip(self, _fip, _cmdline_flag, _dt_node):
        if (not os.path.isfile(_file)):
            print("ERROR: FIP file \"" + _file + "\" not found")
            raise
        try:
            subprocess.check_output([FIPTOOL, "info", _file], stderr=subprocess.STDOUT)
        except:
            print("ERROR: file \"" + _file + "\" is not a FIP file")
            raise
        try:
            out = subprocess.check_output([FIPTOOL, "unpack", "--force", _cmdline_flag, "/dev/null", _file], stderr=subprocess.STDOUT)
            if len(out):
                raise
        except:
            print("ERROR: FIP file \"" + _file + "\" don't have " + _cmdline_flag + " payload")
            raise
        try:
            out = subprocess.check_output(["sh", "-c",
                    FIPTOOL + " unpack --force --fw-config /dev/stdout " + _file + " | " \
                    + DTC + " -I dtb -O dts | " \
                    + SED + " -n '/{$/{h};/load-address/{G;/" + _dt_node + " /{s/>.*//;s/.*0x/0x/;p}}'"]).decode('utf-8')
            return out
        except:
            print("ERROR: cannot extract " + _dt_node + " offset from FIP file \"" + _file + "\"")
            raise

    # get from FIP the load address of optee bl32
    def __get_load_addr_optee_bl32_from_fip(self, _fip):
        global load_add_optee_bl32
        load_add_optee_bl32 = self.__get_load_addr_from_fip(_fip, "--tos-fw", "tos_fw")

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

    # set path for source code
    def __override_elf_src(self, _prj_name, _path, _source_file, _elf_file):
       try:
           s = subprocess.check_output(["sh", "-c", READELF + " --string-dump=.debug_str --string-dump=.debug_line_str " + _elf_file + " 2> /dev/null | grep " + _source_file + "$"]).decode('utf-8')
       except:
           print("Warning: cannot get path of " + _source_file + " from elf file \"" + _elf_file)
           return None
       s = s.split(None)[-1].split(_source_file)[0]
       if s == "" or s == "../":
           gdb.execute("directory " + _path + "/")
       else:
           gdb.execute("set substitute-path " + s + " " + _path + "/")
           gdb.execute("directory " + _path + "/")

    def __init__(self):
        print()
        use_reset = (gdb.convenience_variable("debug_mode") == 0)
        debug_phase = gdb.convenience_variable("debug_phase")
        # TODO: print a summary of attach mode

        try:
            self.__check_user_vars_defined()

            self.__check_ext_tool("readelf", READELF, "--version")
            #if debug_phase != 1 and not use_optee:
            #    self.__check_ext_tool("dtc",     DTC,     "-v")
            #    self.__check_ext_tool("fiptool", FIPTOOL, "help")
            #    self.__check_ext_tool("sed",     SED,     "--version")
            #    self.__get_bl32_offset_from_fip(path_fip)

            if use_reset or debug_phase == 1:
                self.__check_elf_file("TF-A BL2",  path_elf_tf_a_bl2,    "bl2_entrypoint")
            if debug_phase != 1:
                self.__check_elf_file("OP-TEE",    path_elf_op_tee_bl32, "init_tee_runtime")
            if debug_phase == 3:
                self.__check_elf_file("U-Boot",    path_elf_u_boot,      "bootdelay_process")
            if debug_phase == 4:
                self.__check_elf_file("Linux",     path_elf_vmlinux,     "linux_banner")

            self.__check_source_dir("TF-A",   path_source_tf_a,   "common/bl_common.c")
            self.__override_elf_src("TF-A",   path_source_tf_a,   "common/bl_common.c", path_elf_tf_a_bl2)
            if debug_phase != 1:
                self.__check_source_dir("OP-TEE", path_source_op_tee, "core/tee/entry_std.c")
                self.__override_elf_src("OP-TEE", path_source_op_tee, "core/tee/entry_std.c", path_elf_op_tee_bl32)
            if debug_phase == 3:
                self.__check_source_dir("U-Boot", path_source_u_boot, "cmd/version.c")
                self.__override_elf_src("U-Boot", path_source_u_boot, "cmd/version.c", path_elf_u_boot)
            if debug_phase == 4:
                self.__check_source_dir("Linux",  path_source_linux,  "kernel/printk/printk.c")
                self.__override_elf_src("Linux",  path_source_linux,  "kernel/printk/printk.c", path_elf_vmlinux)

        except:
            gdb.execute("set confirm off")
            gdb.execute("quit")
        print()

STM32_gdb()


# after connection with OpenOCD, call in GDB:
#   python STM32_get_target_name()
# to get the name of the target (stm32mp13x or stm32mp15x or stm32mp25x) in GDB variable $target_name
class STM32_get_target_name (gdb.Command):
    def __init__(self):
        default_name = "stm32mp15x"
        try:
            name = gdb.execute("monitor echo -n $_CHIPNAME", False, True)
        except:
            print("Unable to get target name. Default to \'" + default_name + "\'")
            name = default_name

        # name = name.strip()
        if name != "stm32mp13x" and name != "stm32mp15x" and name != "stm32mp25x":
            print("Got unknown target name \'" + name + "\'. Default to \'" + default_name + "\'")
            name = default_name

        gdb.set_convenience_variable("target_name", name)
