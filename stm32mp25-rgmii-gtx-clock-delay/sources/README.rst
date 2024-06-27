=================================
 STM32MP25 RGMII GTX clock delay
=================================

Introdution
===========

RGMII requires a skew between the clock and data signals, as they are
generated simultaneously by the transmitting source. This skew is
usually introduced through PCB trace routing. However, if the actual
skew between the GTX clock and data signals is not large enough, it is
possible to add an extra delay on the GTX clock signal from the
STM32MP2 itself. A tool named 'stm32mp25-rgmii-gtx-clock-delay' was
especially developed for that purpose.

How to install this tool
========================

As of writing this, the tool is not available by default on any
OpenSTLinux images. Therefore, it must be manually installed into the
target root file-system. This can be done directly from the target if
it has an internet connection::

   root@stm32mp25:~# wget TBD --output /usr/bin/stm32mp25-rgmii-gtx-clock-delay

Another way is to transfer the tool from a PC to the target
file-system.  For instance, assuming users can access the target root
file system through /dev/sdXY on their PC::

   PC $> mount /dev/sdXY /mnt
   PC $> wget TBD --output /mnt/usr/bin/stm32mp25-rgmii-gtx-clock-delay
   PC $> chmod +x /mnt/usr/bin/stm32mp25-rgmii-gtx-clock-delay
   PC $> umount /mnt

How to use this tool
====================

The tool 'stm32mp25-rgmii-gtx-clock-delay' can be used to
automatically find the best clock delay and to get or set a clock
delay manually::

  root@stm32mp25:~# stm32mp25-rgmii-gtx-clock-delay
  Handle STM32MP25 RGMII GTX clock delay
  
  Usage: stm32mp25-rgmii-gtx-clock-delay [OPTIONS] <COMMAND>
  
  Commands:
    benchmark  Benchmark all possible RGMII GTX clock delays
    set        Set RGMII GTX clock delay
    get        Get RGMII GTX clock delay
    license    Print license & copyright for this software
    help       Print this message or the help of the given subcommand(s)
  
  Options:
    -v, --verbose...  Increase verbosity level (once = debug, twice = trace)
    -h, --help        Print help
    -V, --version     Print version

Benchmark all possible values
-----------------------------

To automatically find the best clock delay, use the 'benchmark' subcommand::

  root@stm32mp25:~# stm32mp25-rgmii-gtx-clock-delay help benchmark
  Benchmark all possible RGMII GTX clock delays
  
  Usage: stm32mp25-rgmii-gtx-clock-delay benchmark [OPTIONS] --device <DEVICE>
  
  Options:
    -d, --device <DEVICE>
            Device name
    -u, --url <URL>
            Benchmark by fetching data from this URL (recommended size > 100 MiB) [default: https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.4.3.tar.xz]
    -s, --speed-low-limit <SPEED_LOW_LIMIT>
            Skip if transfer rate is below SPEED_LOW_LIMIT bytes/second during more than TIMEOUT seconds [default: "100 kiB"]
    -t, --timeout <TIMEOUT>
            Timemout for SPEED_LOW_LIMIT and for the connection phase [default: 5]
    -h, --help
            Print help

The only required option is -d/--device. If the running system does
not have access to the internet, specifically https://cdn.kernel.org,
then another URL must be specified using the -u/--url option. In that
case, it is strongly recommended to point to a payload which is more
than 100 MiB. Other options have default values that should be
suitable for all cases, and thus, they can be ignored.

Here's a typical example. This can take a couple of minutes, depending
on the network speed::

  root@stm32mp25:~# stm32mp25-rgmii-gtx-clock-delay benchmark --device eth1
  Using URL https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.4.3.tar.xz
  Pass 1/2
  Benchmarking RGMII GTX clock delay = 0.00 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 0.30 nanoseconds... Done in 8.90s; CRC error rate was 1.43% (1363/95451)
  Benchmarking RGMII GTX clock delay = 0.50 nanoseconds... Done in 2.46s; CRC error rate was 1.43% (1362/95164)
  Benchmarking RGMII GTX clock delay = 0.75 nanoseconds... Done in 2.46s; CRC error rate was 1.47% (1395/95162)
  Benchmarking RGMII GTX clock delay = 1.00 nanoseconds... Done in 2.48s; CRC error rate was 1.53% (1460/95165)
  Benchmarking RGMII GTX clock delay = 1.25 nanoseconds... Done in 2.41s; CRC error rate was 1.46% (1393/95162)
  Benchmarking RGMII GTX clock delay = 1.50 nanoseconds... [28] Timeout was reached (Operation too slow. Less than 102400 bytes/sec transferred the last 5 seconds)
  Benchmarking RGMII GTX clock delay = 1.75 nanoseconds... [28] Timeout was reached (Operation too slow. Less than 102400 bytes/sec transferred the last 5 seconds)
  Benchmarking RGMII GTX clock delay = 2.00 nanoseconds... [28] Timeout was reached (Operation too slow. Less than 102400 bytes/sec transferred the last 5 seconds)
  Benchmarking RGMII GTX clock delay = 2.25 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 2.50 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 2.75 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 3.00 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 3.25 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Pass 2/2
  Benchmarking RGMII GTX clock delay = 3.25 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 3.00 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 2.75 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 2.50 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 2.25 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 2.00 nanoseconds... [28] Timeout was reached (Resolving timed out after 5000 milliseconds)
  Benchmarking RGMII GTX clock delay = 1.75 nanoseconds... [28] Timeout was reached (Operation too slow. Less than 102400 bytes/sec transferred the last 5 seconds)
  Benchmarking RGMII GTX clock delay = 1.50 nanoseconds... Done in 4.24s; CRC error rate was 1.51% (1434/95169)
  Benchmarking RGMII GTX clock delay = 1.25 nanoseconds... Done in 2.43s; CRC error rate was 1.36% (1296/95165)
  Benchmarking RGMII GTX clock delay = 1.00 nanoseconds... Done in 2.51s; CRC error rate was 1.53% (1458/95162)
  Benchmarking RGMII GTX clock delay = 0.75 nanoseconds... Done in 2.46s; CRC error rate was 1.50% (1426/95165)
  Benchmarking RGMII GTX clock delay = 0.50 nanoseconds... Done in 2.45s; CRC error rate was 1.52% (1447/95164)
  Benchmarking RGMII GTX clock delay = 0.30 nanoseconds... Done in 9.56s; CRC error rate was 1.43% (1365/95169)
  Benchmarking RGMII GTX clock delay = 0.00 nanoseconds... [28] Timeout was reached (Operation too slow. Less than 102400 bytes/sec transferred the last 5 seconds)
  Best RGMII GTX clock delay is 0.75 ns
  To permanently use this RGMII GTX clock delay, add "st,io-delay = <0x3>;" into following device-tree node(s):
          /soc/pinctrl@44240000/eth2-rgmii-0/pins2
          /soc/pinctrl@44240000/eth2-rgmii-test-1/pins2
          /soc/pinctrl@44240000/eth2-rgmii-test-0/pins1

The most important information is delivered at the end of the
benchmark.  In the previous example, this is this part::

  Best RGMII GTX clock delay is 0.75 ns
  To permanently use this RGMII GTX clock delay, add "st,io-delay = <0x3>;" into following device-tree node(s):
          /soc/pinctrl@44240000/eth2-rgmii-0/pins2
          /soc/pinctrl@44240000/eth2-rgmii-test-1/pins2
          /soc/pinctrl@44240000/eth2-rgmii-test-0/pins1

It shows the best RGMII GTX clock delay, which is 0.75 nanoseconds in
this example, and provides instructions on how to modify the
device-tree to use this value permanently.


Set or get current value
------------------------

This tool can also be used to manually get and set the current RGMII
GTX clock delay::

   root@stm32mp25:~# stm32mp25-rgmii-gtx-clock-delay get --device eth1
   device named "eth1" is known as "eth2" in device-tree
   ↳ its RGMII GTX clock is connected to GPIO F7 (pinctrl@44240000)
     ↳ its delay can be accessed at address 0x44290040 (bits 28-32) in /dev/mem
       ↳ its value is 0x5 (1.25 nanoseconds)

   root@stm32mp25:~# stm32mp25-rgmii-gtx-clock-delay set --device eth1 --clock-delay 0.75
   device named "eth1" is known as "eth2" in device-tree
   ↳ its RGMII GTX clock is connected to GPIO F7 (pinctrl@44240000)
     ↳ its delay can be accessed at address 0x44290040 (bits 28-32) in /dev/mem
       ↳ its value is 0x3 (0.75 nanoseconds)

Potential issues & solutions
============================

"No reliable RGMII GTX clock delay found"
-----------------------------------------

If the tool reports "No reliable RGMII GTX clock delay found", it
indicates that the signal may be too degraded to sustain
megabytes of download.  To avoid this situation, you can specify
a smaller payload using the --url option. However, it may be more
suitable to redesign the PCB traces causing the issue.

"Couldn't resolve host name"
----------------------------

If you are seeing the error message "Couldn't resolve host name", it
could be due to issues with your network settings.  Please check your
network settings (/etc/resolv.conf, https_proxy env. variable, ...)
and make sure that they are configured correctly.

"SSL certificate problem"
-------------------------

If you encounter the error message "SSL certificate problem", it
probably means that some CA certificates are missing on your
system. To fix this, please copy missing certificates into the
directory "/usr/local/share/ca-certificates/", and then run the
command "update-ca-certificates". This should solve this issue.
