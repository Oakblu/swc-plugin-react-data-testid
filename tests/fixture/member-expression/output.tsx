function Dialog() {
    return <Modal data-testid="Dialog.Modal">
      <Modal.Header data-testid="Dialog.Header">Title</Modal.Header>
      <Modal.Body data-testid="Dialog.Body">Content</Modal.Body>
      <Modal.Footer data-testid="Dialog.Footer">
        <button data-testid="Dialog.button">Close</button>
      </Modal.Footer>
    </Modal>;
}
