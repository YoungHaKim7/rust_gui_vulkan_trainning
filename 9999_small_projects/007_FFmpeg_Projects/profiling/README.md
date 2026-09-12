# process 분석(`perf`)

```bash
$ sudo perf top -p 6551



$ sudo perf record -g -p 6551 -- sleep 5
[ perf record: Woken up 8 times to write data ]
[ perf record: Captured and wrote 2.177 MB perf.data (20018 samples) ]


$ ls
perf.data

$ sudo perf report
```

# ps분

```bash
$ ps -eo pid,ppid,%cpu,%mem,etime,cmd --sort=-%cpu | head -20
    PID    PPID %CPU %MEM     ELAPSED CMD
   6551    6540 99.9  7.2       01:20 /ffmpeg_rs/target/debug/deps/ffmpeg_rs-92bbeda044d74837 --test-threads=1
   3374    1538 16.0  0.8       05:44 /usr/lib64/firefox/firefox
   5176    3748  9.1  0.4       05:05 /usr/lib64/firefox/firefox -contentproc -isForBrowser -prefsHandle 0:51492 -prefMapHandle 1:295428 -jsInitHandle 2:160936 -parentBuildID 20260903215306 -sandboxReporter 3 -chrootClient 4 -ipcHandle 5 -initialChannelId {d0de65d4-cc92-4ac4-a510-9f5731f4137b} -parentPid 3374 -crashHelperPid 3594 -crashHelper 6 -crashReporter 7 -greomni /usr/lib64/firefox/omni.ja -appomni /usr/lib64/firefox/browser/omni.ja -appDir /usr/lib64/firefox/browser 14 tab


$ ps -fp 6540
UID          PID    PPID  C STIME TTY          TIME CMD
gy          6540    6410  0 19:49 pts/0    00:00:00 /home/gy/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo test -- --test-threads=1


$ pstree -aps 6551
systemd,1 --switched-root --system --deserialize=59
  └─systemd,1461 --user
      └─ghostty,6360
          └─fish,6410
              └─cargo,6540 test -- --test-threads=1
                  └─ffmpeg_rs-92bbe,6551 --test-threads=1
                      └─{ffmpeg_rs-92bbe},6720

$ ps -L -p 6551 -o pid,tid,psr,%cpu,stat,etime,comm
    PID     TID PSR %CPU STAT     ELAPSED COMMAND
   6551    6551   0  0.0 Sl+        04:47 ffmpeg_rs-92bbe
   6551    6720   3 99.9 Rl+        04:47 format::nut::mu
```
