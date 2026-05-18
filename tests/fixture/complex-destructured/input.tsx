const { Component: RenamedComponent } = React;
const [FirstComponent, SecondComponent] = components;

const Wrapper = () => {
  return (
    <div>
      <RenamedComponent />
      <FirstComponent />
    </div>
  )
}
