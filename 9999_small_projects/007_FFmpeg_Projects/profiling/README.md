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
