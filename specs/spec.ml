type 'a set = Set of 'a

module type Logic_unit_relations = sig
  type logical_unit = LU

  (* Recording the super -> sub ordering of a single path traced through the
     labrynth of requirements *)
  type path =
    | Derivation of logical_unit list
    | Allocation of logical_unit list

  (** {1} Diachronic relations

      Specification/implementation relations between logical units in terms of
      entailment and requisition. *)

  val subsequents : logical_unit -> logical_unit set
  (** [subsequents lu] is the set of logical units that follow directly from
      [lu]. *)

  val antecedent : logical_unit -> logical_unit option
  (** [antecedent lu] is [Some lu'] if [lu'] is the immediate antecedent of [lu],
      or [None] if [lu] is primordeal. *)

  (** {1} Synchronic relations

      Abstraction/specificaiton Relations between logical units in terms of the
      generality.  *)

  val genus : logical_unit -> logical_unit option
  (** [genus lu] is [Some lu'] if [lu'] is next most general specifiction above
      [lu] in the order of abstraction. If [lu] is a most general logical unit,
      then [genus lu] is [None]. *)

  val specifications : logical_unit -> logical_unit set
  (** [specifications lu] are the set of logical units that refine [lu] by
      specifying a more specific requirement.

      {|
      forall (x : logical_unit).
        (exists y: logical_unit. Set.mem x (specifications y)) <=> x = (genus y)
      |} *)

  type artifact = logical_unit set

  val requirements : artifact -> artifact set
  (** [requirements a] is the set of artifacts that realize [a]. That is, the
      set of artifacts which contain logical units [lus] such that there exists
      a logical unit [l] in [a] where [Set.mem (antecedent l) lus]. *)

  type realizers = artifact -> artifact set

  val realizers : realizers
  (** [realizer a] is an artifact that realizes or satisfies a requirement of
      a logical unit found in [a]. *)

  val verfied_realizations : artifact -> realizers set
  (** [verified_realizations a] is the set of relizations relations which are
      verified by the tests or proofs supplied by [a]. *)

  type repo = artifact set

  val specifiers : repo -> repo set
  (** [specifiers r] are the set of repos that dictate specifications refiend or
      implemented by [r]. *)
end

module type Trace = sig
  (* (Relatively) simple elements *)
  type element

  type 'a set

  type thing = element set

  val influence : thing -> thing -> thing
  (** [influence t1 t2] is [t2'] where [t2'] is [t2] after
      being changed/formed/or altered by some aspect of [t1] *)

  val influences : thing -> thing set
  (** [influences t] are the set of things that have influenced [t] *)

  type trace = Trace of thing * thing
end

exception TODO

exception DONE

let set_filter : ('a -> bool) -> 'a set -> 'a set = raise TODO

let set_is_empty : 'a set -> bool = raise TODO

type path
(** {1 Ambiet structures} *)

(** {1 Components} *)

(** {3 Implements}

    [LOGICAL-UNIT.1::OSPEC.1] *)
module Logical_unit = struct
  (**
     [TRC-TAG.1::SYNTAX.1::OSPEC.1] *)
  module Id : sig
    type t
  end = struct
    type t = string
  end

  module Spec = struct
    type written = [ `Written ]
    (** [LOGICAL-UNIT.1::SPEC.1::WRITTEN.1::OSPEC.1] *)

    type executable =
      [ written
      | `Executable
      ]
    (** [LOGICAL-UNIT.1::SPEC.1::EXECUTABLE.1::OSPEC.1] *)

    type t =
      [ written
      | executable
      ]
    (** [LOGICAL-UNIT.1::SPEC.1] *)
  end

  type kind =
    | Spec of Spec.t
    | Implementation  (** [LOGICAL-UNIT.1::IMPLEMENTATION.1::KIND.1] *)
    | Verification  (** [LOGICAL-UNIT.1::VERIFICATION.1::KIND.1] *)

  (** |TRC-GITHUB-REF.1::IMPL.1::LOC.1|*)
  module Source = struct
    type t =
      { repo : string option
      ; file : path option
      ; line : int option
      }
  end

  type t =
    { id : Id.t
    ; kind : kind
    ; content : string
    ; references : Id.t list
    ; source : Source.t
    }

  (** Gives the version of the unit itself
      E.g., [assert (version |FOO.1::BAR.1|) = 1;] *)
  let version : t -> int = raise TODO
end

(** |TRC-GITHUB-REF.1::IMPL.1::REPO.1|*)
module Repo = struct
  type local =
    { name : string option
    ; default_upstream : string option
    ; default_branch : string option
    }

  type remote =
    { url : string
    ; default_branch : string option
    }

  type location =
    | Local of local
    | Remote of remote

  type t =
    { artifacts : Artifact.t set
    ; location : location
    }

  let artifacts : t -> Artifact.t set = raise TODO

  let logical_units : t -> Logical_unit.t set = raise TODO

  let sync : t -> t = raise TODO
end

(** |TRC-GRAPH.1::BUILD.1::CONTEXT.1|
    : A [Context] includes all artifacts to be included in for tracability in
      its registry. *)
module Context = struct
  type name = string

  type t =
    { name : name
    ; repos : Repo.t set
    }

  let list_repos : t -> Repo.t set = raise TODO

  let add_repo : t -> Repo.t -> t = raise TODO

  let remove_repo : t -> Repo.t -> t = raise TODO

  (** List all specs in the context *)
  let list_units : t -> Logical_unit.t set = raise TODO

  let add_unit : t -> Repo.t -> t = raise TODO

  let remove_unit : t -> Repo.t -> t = raise TODO

  let sync : t -> t = raise TODO
end

(** Interface to context management *)
module Context_switch = struct
  type t =
    { current : Context.t
    ; contexts : Context.t set
    }

  let list : Context.t set = raise TODO

  let change : Context.name -> t -> t = raise TODO

  let new_ : Context.name -> t -> t = raise TODO
end

(** |TRC-GRAPH.1::BUILD.1::CONTEXT.1::DB.1| *)
module Db = struct
  type t =
    { repositories : Repo.t set
    ; logical_units : Logical_unit.t set
    ; contexts : Context.t set
    }

  type err = Duplicate of Logical_unit.Id.t

  (** |{TRC-UNIQ.1::DUPS.1,TRC-GRAPH.1:CI.1}::ADD.1| *)
  let add_unit : t -> Logical_unit.t -> (t, err) result = raise TODO

  let find_all_units : t -> (Logical_unit.t -> bool) -> Logical_unit.t set =
    raise TODO

  (** |TRC-MISS.1::ANALYSIS.1::ORPHANS.1| *)
  let orphans : t -> Logical_unit.t set = raise TODO

  (** |TRC-MISS.1::ANALYSIS.1::CHILDLESS.1| *)
  let childless : t -> Logical_unit.t set = raise TODO

  (** |TRC-MISS.1::OUTDATED.1::PARENTS.1| *)
  let parent : t -> Logical_unit.t -> Logical_unit.t = raise TODO

  (** Transitive closure of parent relation *)
  let ancestors : t -> Logical_unit.t -> Logical_unit.t set = raise TODO

  (** |TRC-MISS.1::OUTDATED.1::CHILDREN.1| *)
  let children : t -> Logical_unit.t -> Logical_unit.t set = raise TODO

  (** Transitive closure of children relation *)
  let descendants : t -> Logical_unit.t -> Logical_unit.t set = raise TODO

  (** |TRC-MISS.1::OUTDATED.1::FIND.1| *)
  let outdated : t -> Logical_unit.t set =
   fun db ->
    let is_outdated lu =
      let version_of_lu = Logical_unit.version lu in
      let newer_parents =
        ancestors db lu
        |> set_filter (fun parent ->
               Logical_unit.version parent > version_of_lu)
      in
      not (set_is_empty newer_parents)
    in
    find_all_units db is_outdated

  let unit_source : t -> Logical_unit.t -> Artifact.t = raise TODO
end

module Report = struct
  type t = string

  (** |TRC-MISS.1::ANALYSIS.1::REPORT.1| *)
  let report : Db.t -> t = raise TODO
end
