# BellMetal
A Rust library for efficiently proving and generating change ringing touches.

I would hope that real bells have less rust than this repo!

This library is designed mainly with my own use in mind.  It has been pivotal in the composition of many of my more recent compositions, including what I believe is the first time where a peal composition in its entirety has had rotational as well as palindromic symmetry: [5004 9-Spliced Royal by Ben White-Horne](https://complib.org/composition/65034).  It also helped me improve some old compositions, such as this newer cyclic variable-treble QP of Plain Bob Triples, with a lot of 4-bell runs: [1274 Plain Bob Triples by Ben White-Horne](https://complib.org/composition/61698).

## Notable Features
- Plugable music scoring system
- Pretty printing of touches, including multi-column view, music highlighting (determined by the music system you're using), method names and ruleoffs, calls, and a coloured summary line like in CompLib.
- Falseness display in the pretty printing that displays groups of adjacent false changes with the same colours.
- Support for discontinous touches (i.e. ones in which bells may occasionally jump more than one place), and methods which only have a start and end of each lead.
- Proving system that can prove multi-part peals by only generating one part.
- An iterator system to allow touch generation and proving without generating the entire composition on the heap.

## Example screenshots
