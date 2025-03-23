(* Types *)

type hash_input = bytes
type hash_output = bytes {length hash_output = 256 /\ well_distributed hash_output}

(* Hash Function *)

val hash: hash_input -> hash_output
let hash input =
  let output = hash_impl input in
  assert_norm (pow2 256 > output);
  output

(* Properties *)

val hash_deterministic: input:hash_input -> output:hash_output{hash input == output}
let hash_deterministic input =
  let output = hash input in
  assert (output == hash input)

val hash_uniform_distribution: output:hash_output -> prop
let hash_uniform_distribution output =
  well_distributed output

val hash_avalanche: input1:hash_input -> input2:hash_input{hamming_distance input1 input2 <= 1} ->
  Lemma (requires True)
        (ensures hamming_distance (hash input1) (hash input2) >= 128)
        [SMTPat (hash input1); SMTPat (hash input2)]

val hash_collision_resistant: input1:hash_input -> input2:hash_input{input1 <> input2} ->
  Lemma (requires True)
        (ensures hash input1 <> hash input2)
        [SMTPat (hash input1); SMTPat (hash input2)]

val hash_one_way: output:hash_output -> input:hash_input{hash input == output} ->
  Lemma (requires True)
        (ensures (forall (adversary:hash_input -> hash_input). adversary output <> input))
        [SMTPat output]

(* Preconditions and Postconditions *)

val hash_impl: input:hash_input -> output:hash_output{hash_deterministic input output /\ hash_uniform_distribution output}
let hash_impl input =
  (* Implementation details *)
  ...

(* Invariants *)

type hash_state = {
  state: bytes;
  invariant well_formed_state state
}

val well_formed_state: state:bytes -> prop
let well_formed_state state = ...

(* Security Properties *)

val hash_confidentiality: input1:hash_input -> input2:hash_input{input1 <> input2} ->
  Lemma (requires True)
        (ensures hash input1 <> hash input2)
        [SMTPat (hash input1); SMTPat (hash input2)]

val hash_integrity: input:hash_input -> output:hash_output{hash input == output} ->
  Lemma (requires True)
        (ensures (forall (adversary:hash_output -> hash_output). adversary output <> output))
        [SMTPat output]

(* Resource Usage Constraints *)

val hash_time_complexity: input:hash_input -> Ghost nat
let hash_time_complexity input =
  let output = hash input in
  O (length input)

val hash_space_complexity: input:hash_input -> Ghost nat
let hash_space_complexity input =
  let output = hash input in
  O (length input)
