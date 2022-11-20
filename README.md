Step to reproduce:

Start the producer, check the signal is sent with

```sh
busctl --user monitor ludo_ic.daemon.producer
```

Then start the listener **without** zbus logs.

~~Open a shell and send some dbus command to the listener (I'm not sure if it is really needed)~~
```sh
while [ 1 ]; do busctl --user call ludo_ic.daemon.other /ludo_ic/daemon/other ludo_ic.daemon.other SayHello; sleep 0.3; done
```

I reproduce it without the busctl command, but the adpator is mandatory. Without it th issue isn't reproduced.

Just launch a `RUST_LOG=debug cargo run` command in another project to be sure that all CPUs are stressed.

On my machine (i7-1165G7), after few seconds the red "error" log isn't printed anymore, that's the moment when the signal is received by zbus but the proxy does not receive it.

After 5 seconds without signal, the process is exited.
