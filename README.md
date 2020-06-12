# BellMetal
A Rust library for efficiently proving and generating change ringing touches.

I would hope that real bells have less rust than this repo!

This library is designed mainly with my own use in mind.  It has been pivotal in the creation of many of my more recent compositions, including what I believe is the first time where a peal composition in its entirety has had rotational as well as palindromic symmetry (as well as being the record highest-music-scoring 9-part of Royal): [5004 9-Spliced Royal by Ben White-Horne](https://complib.org/composition/65034).  It also helped me improve some old compositions, such as this newer cyclic variable-treble quarter peal of Plain Bob Triples, with 72 of the 96 possible 4-bell runs: [1274 Plain Bob Triples by Ben White-Horne](https://complib.org/composition/61698).  I have started work on Python bindings which will hopefully have both the flexibilty of Python and the speed of Rust, as well as making the library easier to use since it doesn't require learning a fairly challenging systems programming language.

## Notable Features
- Plugable music scoring system
- Pretty printing of touches, including multi-column view, music highlighting (determined by the music system you're using), method names and ruleoffs, calls, and a coloured summary line like in CompLib.
- Falseness display in the pretty printing that displays groups of adjacent false changes with the same colours.
- Support for discontinous touches (i.e. ones in which bells may occasionally jump more than one place), and methods which only have a start and end of each lead.
- Proving system that can prove multi-part peals by only generating one part.
- An iterator system to allow touch generation and proving without generating the entire composition on the heap.

## Example screenshots
![Cyclic Double Royal](https://raw.githubusercontent.com/Kneasle/bellmetal/master/screenshots/cyclic-double-royal.png)

A pretty-printing of 1/36th of the cylic rotational peal mentioned above.  This is what I saw whilst designing/converting methods for it.

![Very False](https://raw.githubusercontent.com/Kneasle/bellmetal/master/screenshots/very-false-8-part.png)

A pretty-printing of a short touch of spliced Surprise Major, with the proving done as though it was the start of each part of a cyclic 8-part.  Demonstrates the false row grouping, as well as how chaotic the falseness of cyclic 8-parts is.

![Snow Tiger Delight Maximus](https://raw.githubusercontent.com/Kneasle/bellmetal/master/screenshots/snow-tiger.png)
An example of the course of Maximus with Coursing order 685324 (or E976853240T in full), which has 3 different sets of 4-bell runs.  Used in this composition: [5136 Snow Tiger Delight Maximus by Ben White-Horne](https://complib.org/composition/63424), which I believe has the most 4-bell runs out of any peal of Snow Tiger to date.
