const { Component: RenamedComponent } = React;
const [FirstComponent, SecondComponent] = components;
const Wrapper = ()=>{
    return <div data-testid="Wrapper.div">
      <RenamedComponent data-testid="Wrapper.RenamedComponent"/>
      <FirstComponent data-testid="Wrapper.FirstComponent"/>
    </div>;
};
