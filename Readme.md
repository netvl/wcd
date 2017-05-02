# Wallpaper change daemon

This program is client-server application, where the server works as a daemon and regularly
executes a shell command, going through a playlist of files. If the command is something like
`feh --bg-fill {}`, and the playlist contains images, then it will result in a change of
desktop background after the next configured interval passes. Thus you can configure a list
of wallpapers, set the interval and then enjoy a new background picture every hour or so :)

wcd is mostly intended for users of lightweight desktop environments on Linux, like i3 or Awesome
or Openbox or whatever. However, the program is cross-platform, and if you can change the
wallpaper through some shell command on your system, you can use it.

The daemon and the client programs are combined in a single binary. With the command-line
client you can trigger immediate wallpaper change, switch playlists, get current status etc.
For example, you can bind a key shortcut in your window manager to force wallpaper change
when you want it. Or maybe to switch between SFW and NSFW playlists, if the situation needs it :)

## Installation

wcd uses Cargo, so first install the Rust compiler, and then run the following command to 
install the binary:

```
% cargo install --git https://github.com/netvl/wcd wcd
```

This command will install the binary to wherever your Cargo binaries directory is located
(usually it is `~/.cargo/bin`), so in order to run them, you need to make sure that this
directory is added to your `PATH`.

While wcd does not use compiler features and therefore could theoretically be compiled with
the stable Rust compiler, I use nightly Rust for development. I'll probably add a Travis
configuration with multiple Rust versions in future to track this.

Note that wcd depends on [nanomsg](http://nanomsg.org/), so you need to have its libraries
available for the linker. Currently `nanomsg-rs` does not use pkg-config to find libraries,
so you need to make nanomsg shared library available in the library search path (`PATH` in 
Windows, `LD_LIBRARY_PATH` in Linux, `DYLD_LIBRARY_PATH` in OS X). On Linux it usually means
installing the respective package with your package manager.

## Configuration

Both the daemon and the client use the same configuration file. By default it is assumed to be
`~/.config/wcd/config.toml`. Maybe in the future I'll change it to consider `$XDG_CONFIG_DIR`
or other platform-specific configuration directory, but for now the default is harcoded.
You can use `-c` command line option to choose another config file.

The configuration file is a TOML document. Here is an example:

```toml
[common]
endpoint = "ipc:///tmp/wcd.ipc"

[server]
default_playlist = "default"
watch = "1 minute"
# watch = "disabled"

[server.defaults]
mode = "random"  # or "sequential"
change_every = "1 hour"
trigger_on_select = true
use_last_on_select = true
command = ["feh", "--bg-fill", "{}"]

[server.playlists.default]
files = ["~/downloads/pic.jpg", "/usr/share/whatever/picture.png"]
directories = ["~/pictures/wallpapers", "/mnt/storage/pictures"]
change_every = "2 hours"

[server.playlists.sfw]
directories = ["~/pictures/safe"]
mode = "sequential"
use_last_on_select = false
```

`[common]` section contains a single option, `endpoint`, which is needed to set up the connection
between the daemon and the client. wcd uses [nanomsg](http://nanomsg.org/) for client-server
communication, and so it supports all the transports as they are described 
[in its documentation](http://nanomsg.org/v0.8/nanomsg.7.html) (scroll to the bottom). Because
usually both the server and the client are located on the same machine, it makes sense to
use the `ipc` transport which maps to a UNIX socket. I haven't tested it on Windows so I can't say
whether `ipc` works there, but the `tcp` transport should work anywhere. Because the same
configuration file is shared by both the daemon and the client, this address needs to be configured
only once.

The `[server]` section contains global server configuration options.
* `default_playlist` specifies the playlist which will be used immediately after the daemon
  starts up. Afterwards the playlist can be changed to one of the other configured playlists
  with the client; more on it below.
* `watch` defines the playlist refresh behavior. If you add new images to the directories
  configured for playlists, wcd will detect them and incorporate them into the respective
  playlists automatically. This option determines the interval between full directory rescans.
  Put `"disabled"` here to disable regular polls. Either way, you can always force a refresh
  with the client. In a future version I may add an inotify-based (or whatever cross-platform file
  watch library I will be able to find) watch.

`[server.default]` controls the default options for all playlists. Each option in this
section may be overridden inside a playlist. There is no way to configure default files
and directories, however.
* `mode` determines the way the playlist will be "executed". When wcd starts,
  it compiles a list of all appropriate files for each playlist. If this option is set
  to `"random"`, then this list will be shuffled. Moreover, the reshuffle will occur after
  the playlist ends. `"sequential"` means that the order of files will be fixed and equal
  to the order reported by the file system when directories are scanned.
* `change_every` sets the interval after which the next item in the playlist
  should be used. You can use any unit from nanoseconds up to days (including abbreviations
  like `us`, `micros`, `mins`, etc., and either singular or plural form: `second` or `seconds`),
  however, wcd's scheduler resolution is 1 second so intervals smaller than this are useless.
* `trigger_on_select` determines whether the wallpaper should be changed if you has switched
  to this playlist from some other playlist. It works in conjuction with `use_last_on_select`
  option, see below. If this option is set to `false`, then making this playlist current
  will not make wcd execute any commands (unless the interval for this playlist is smaller than
  the one for the previous playlist, and it has already passed since the last wallpaper switch).
  If set to `true`, this option's behavior is determined by the `use_last_on_select` option.
* `use_last_on_select` is only considered if `trigger_on_select` is `true`. If this
  option is `true`, then the last used wallpaper will be restored after this playlist is selected.
  The timer, however, will be reset, so the next change will happen after the duration configured
  for this playlist. If this option is `false`, then the next image will be used, with the usual
  effect on the timer.
* `command` sets up the command which should be executed to change the wallpaper. The
  command is a list of strings, where the first item is the command name and all other items
  are passed to the command as arguments. Each occurence of `{}` placeholder in options
  will be replaced with the image file name, however, `{}` only works as a whole argument:
  `"-c={}"` means `-c={}`, not `-c=<file name>`. It is an error for this option to be an
  empty list, as well as to have no placeholders among the arguments.

All of these options are optional for the defaults section. If they are absent here, they must
be configured for each playlist separately; it is an error if any of them, except
`trigger_on_select` and `use_last_on_select`, are not set at least in one place.
`trigger_on_select` and `use_last_on_select`, if absent, are assumed to be `true`.

Each of `[server.playlists.<name>]` section configures a playlist named `<name>`. These sections
may contain the same options as the `[server.defaults]` section (and if present, they will take preference
over the defaults), and also two additional options are available:
* `files` is a list of files which must be included into the playlist.
* `directories` is a list of directories which should be scanned for image files. Currently
  wcd determines whether a file is an image by extension, and it understands `jpg`, `jpeg`
  and `png` extensions, both in lower, upper or mixed case. These directories will also be
  rescanned automatically if watch interval is configured or manually when requested through
  the client.

Paths specified in `files` or `directories` lists may start with `~/`. These paths are resolved
against the home directory of the user running wcd.

## Command-line interface

The daemon should be started by invoking `wcd daemon`. It will be started in the foreground, so if you
want to start it as a service, you should use a service manager specific for your OS (e.g. systemd on Linux).

Other subcommands, like `wcd status`, belong to the client and allow to interact with the daemon. `wcd --help`
provides the entire list of subcommands with their descriptions, so they won't be covered here.

## Verbosity

`wcd` understand a `-v` switch which can be passed zero, one or two times, with each passed option increasing the
verbosity of the daemon output. For client subcommands `-v` may also result in extra output except that of the
executed command. These options are useful for debugging.

## Future features

In no particular order:

* Add support for persisting the state of the daemon between reloads.
* Add support for printing out wallpapers playlists and selecting a particular wallpaper.
* Create a web UI for controlling the daemon.

## License

This program is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed 
as above, without any additional terms or conditions.
