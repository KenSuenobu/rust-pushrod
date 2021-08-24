# rust-pushrod

Pushrod is a library that uses the SDL2 library to interact with the operating system, and uses
OpenGL for the drawing layer.

## Summary

The original version of the library was a callback-based library, but this has changed.  The design
philosophy was that callbacks were a better design, however, this uncovered several design flaws.  First,
passing in mutable references to the top-level object that caused an event was almost impossible without
a series of cast objects.  Second, performance suffered as a result of the custom code that was
written to support said functionality.

After researching other libraries, it became obvious that using an event-based system was a better
approach.  Therefore, Pushrod now uses an event-based system to handle messaging.

The benefits to this are great - the main benefits being that each window now has its own event handler:
every window that displays a new dialog contains its own window event handler.  This allows for global
access to the widget store, the containing object, and the event handler.

## Installing on Mac OS X

Install Brew.  [Go here to get instructions on how to install.](https://www.brew.sh)

Once installed, run the following commands:

```shell
brew update
brew upgrade
brew install sdl2 sdl2_image sdl2_ttf
```

Then modify your .profile and add:

```shell
export LIBRARY_PATH="$LIBRARY_PATH:/opt/homebrew/lib"
```

And you should be able to not only build the application, but you should be able to run the
examples in the examples directory.

