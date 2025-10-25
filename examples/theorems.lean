-- Theorem examples

-- Axiom of function extensionality
axiom funext (α β : Type) (f g : α -> β) :
  (forall x : α, f x = g x) -> f = g

-- Identity is its own inverse
theorem id_inverse (α : Type) :
  comp α α α (id α) (id α) = id α := by
  funext

-- Constant ignores second argument
theorem const_ignores (α β : Type) (x : α) (y z : β) :
  const α β x y = const α β x z := by
  refl

-- Double negation (classical logic would be needed for full proof)
axiom double_neg (P : Prop) : not (not P) -> P

-- Modus ponens
theorem modus_ponens (P Q : Prop) (h1 : P) (h2 : P -> Q) : Q :=
  h2 h1
