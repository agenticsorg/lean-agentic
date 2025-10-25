-- Inductive type examples

-- Natural numbers
inductive Nat where
  | zero : Nat
  | succ (n : Nat) : Nat

-- Boolean
inductive Bool where
  | false : Bool
  | true : Bool

-- Option type
inductive Option (α : Type) where
  | none : Option α
  | some (val : α) : Option α

-- Lists
inductive List (α : Type) where
  | nil : List α
  | cons (head : α) (tail : List α) : List α

-- Binary trees
inductive Tree (α : Type) where
  | leaf : Tree α
  | node (left : Tree α) (value : α) (right : Tree α) : Tree α

-- Equality
inductive Eq (α : Type) (a : α) where
  | refl : Eq α a a
