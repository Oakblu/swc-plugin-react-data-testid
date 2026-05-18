const config = {
  component: () => <div>No parent declarator</div>
}

someFunction(() => <span>Argument function</span>)

const [, secondComponent] = [null, () => <div>Array destructured</div>]
