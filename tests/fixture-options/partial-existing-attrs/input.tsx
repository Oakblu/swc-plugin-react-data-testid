// With attributes: ["data-testid", "data-cy"], each attribute is injected
// independently. If one already exists, only the missing ones are added.
//
// Also reveals insertion-order asymmetry: on a fresh element both attrs are
// inserted at position 0 in loop order, so the last-processed attr ends up
// first in the output. On an element where one attr already exists, the
// newly inserted attr lands at position 0 ahead of the existing one.

function Form() {
  return (
    <form>
      <input data-cy="form-input" type="text" />
      <button data-testid="form-submit">Submit</button>
      <label>Email</label>
    </form>
  );
}
