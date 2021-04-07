# Terminology

## Essential concepts

<span id="TERMINOLOGY.1" class="lu-tag">|TERMINOLOGY.1|</a>
: > from Latin terminus "end, boundary line," in Medieval Latin "expression,
  definition," related to termen "boundary, end"
  ([etymonline](https://www.etymonline.com/word/term#etymonline_v_10648))
: Terminology delimits the bounds of signification, and describes the (relative)
  terminus for communicating with signs.
: We take terminology as a semiotic (de-reified) counterpart to the [reference
  models](https://en.wikipedia.org/wiki/Reference_model) used in OOP and other
  realist requirement tracing frameworks.

<span id="CONTEXT.1" class="lu-tag">|CONTEXT.1|</a>
: A context gathers together the artifacts involved in an enterprise.
: > from Latin contextus "a joining together," originally past participle of
  contexere "to weave together," from assimilated form of com "with, together"
  (see con-) + texere "to weave, to make"  ([etymonline](https://www.etymonline.com/search?q=context))
: Ideally a context is grounded in a [TERMINOLOGY.1][] and composed of all the
  intricate [traces][TRACE.1] that determine the relations between these
  artifacts. The traces should weave together all the influences from inception,
  to specification, to implementation, to documentation, to revision and
  iteration.
: At minimum, a context is a registry of [repositories][REPOSITORY.1].

|LOGICAL-UNIT.1|
: The elementary components of a context, between which traces are to be
  captured, are called "logical units".
:
: We distinguish 3 basic kinds of logical units:

  - [LOGICAL-UNIT.1::SPEC.1]
  - [LOGICAL-UNIT.1::IMPLEMENTATION.1]
  - [LOGICAL-UNIT.1::VERIFICATION.1]

|LOGICAL-UNIT.1::SPEC.1|
: "A  **specification**  is  a  written  description  of  what  a  system  is
  supposed  to  do" (Lamport, [Specifying Systems][]).
:
: We further divide specs into

  - [LOGICAL-UNIT.1::SPEC.1::WRITTEN.1]
  - [LOGICAL-UNIT.1::SPEC.1::EXEUCTABLE.1]

|LOGICAL-UNIT.1::SPEC.1::WRITTEN.1|
: A logical unit is a **written spec** if it is a recorded specification of some
  properties of a system. A written spec communicates requirements between
  humans, but may not be executable by machines. 

These are examples of non-executable written spec units:
  
- A paragraph of an natural language document
- A linear temporal logic formula
- A ticket in a project management tool
- A diagram

*Definition of terms* are special cases of written specs. These are recorded
specifications of the properties of key words within the system of communication
used to establish the context of development and operations. The 5 logical units
in this terminology are examples.

|LOGICAL-UNIT.1::SPEC.1::EXEUCTABLE.1|
: A logical unit that is a [written spec][LOGICAL-UNIT.1::SPEC.1::WRITTEN.1] is
  also an **executable spec** if it can be executed by a machine for use in
  verification, implementation, reference, etc.
  
These are examples of executable spec units: 

- A TLA+ operator
- A Maude equation
- An Alloy predicate

|LOGICAL-UNIT.1::IMPLEMENTATION.1|
: A logical unit is an **implementation** if it implements or contributes to
  implementing some functionality or property of the system. 
  
These are examples of implementation units:

- A constant definition
- A function declaration 
- A module
- A more specific specification that describes how a more general specification
  is to be implemented (see [TRC-IMPL.1::PREFIX.1]). Thus,
  [LOGICAL-UNIT.1::IMPLEMENTATION.1] is an implementation of [LOGICAL-UNIT.1].

|LOGICAL-UNIT.1::VERIFICATION.1|
: A logical unit is a **verification*** if it verifies that an
  [implementation][LOGICAL-UNIT.1::IMPLEMENTATION.1] satisfies (or contributes
  to satisfying) a property of the system. 

These are examples of verification units:

- A unit test
- A model based test
- A type signature
- A module interface
- An integration tests

Verification units generally serve to verify that the
[implementation][LOGICAL-UNIT.1::IMPLEMENTATION.1] satisfies or ensures a
property recorded in a [written][LOGICAL-UNIT::SPEC.1::WRITTEN.1] or
[executable][LOGICAL-UNIT::SPEC.1::EXEUCTABLE.1] spec.

**Note:** These distinctions are (1) relational and (2) not mutually exclusive.

(1) E.g., a unit of verification is determined relative to the system whose
    properties it verifies and the implementation unit it exercises. A random
    test that fires but doesn't touch any external functions is not a
    verification unit.
(2) E.g., function in a dependently typed language can be, simultaneously, a
    specification unit, an implementation unit, and a verification unit.


|TRACE.1|
: TODO Trace back to requirements tracing doc.

|REPOSITORY.1|
: TODO

|USER-AUTHOR.1|
: A user of an open source text (including a program) who is also empowered and
  encouraged to alter and transform the text.

|WORKSITE.1|
: A worksite is place where work is performed. Worksites include

  - Local directories holding projects (e.g., a local git repo of software a UA
    is implementing).
  - An office building or room.
  - A chat server (such as Slack or Zulip)
  - A software [forge](https://en.wikipedia.org/wiki/Forge_(software)), such as
    GitHub, GitLab, or [Radicle](https://radicle.xyz/).

[Specifying Systems]: https://www.microsoft.com/en-us/research/publication/specifying-systems-the-tla-language-and-tools-for-hardware-and-software-engineers/?from=http%3A%2F%2Fresearch.microsoft.com%2Fusers%2Flamport%2Ftla%2Fbook.html

## Abbreviations

UA
: [USER-AUTHOR.1][]

[CONTEXT.1]: #CONTEXT.1
[TERMINOLOGY.1]: #TERMINOLOGY.1
[REPOSITORY.1]: #REPOSITORY.1
[TRACE.1]: #TRACE.1
[USER-AUTHOR.1]: #USER-AUTHOR.1

