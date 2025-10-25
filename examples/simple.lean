-- Simple Lean examples to test the elaborator

-- Identity function
def id (α : Type) (x : α) : α := x

-- Constant function
def const (α β : Type) (x : α) (y : β) : α := x

-- Function composition
def comp (α β γ : Type) (g : β -> γ) (f : α -> β) (x : α) : γ := g (f x)

-- Natural number addition (recursive definition)
def add (n m : Nat) : Nat :=
  match n with
  | 0 => m
  | succ n' => succ (add n' m)

-- Boolean negation
def not (b : Bool) : Bool :=
  match b with
  | true => false
  | false => true

-- Lambda expressions
def double := fun (n : Nat) => add n n

-- Let bindings
def quadruple (n : Nat) : Nat :=
  let d := double n in
  add d d
