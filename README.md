Tequiz
======

```
merge(tetris, quiz)
```

Tetris was the very first programming project I finished back in 2002-2003.
Back then I used C# and it was a GUI application running in Windows.

Recently I watched the movie [Tetris]. When I saw tetrominoes are rendered by
pair of `[]`, I said: Beautiful.

So, after 20 years, I decided to implement the game one more time. This time I
make it a TUI application, and of course, each tetromino is rendered as `[]` by
default:)

However, worth noting that:

> The game has monochrome graphics, and in the first revision of the game, the
> blocks in the tetrominos are represented by a pair of delete/rubout
> characters (character code 177); however, the rendering of this character
> code as a rectangle was a feature specific to the Soviet clone machines, an
> actual PDP-11 would instead display nothing. A later revision was made where
> the blocks are represented by a pair of square brackets instead.
>
> -- <cite>https://tetris.wiki/Tetris_(Electronika_60)</cite>


[Tetris]: https://en.wikipedia.org/wiki/Tetris_(film)
