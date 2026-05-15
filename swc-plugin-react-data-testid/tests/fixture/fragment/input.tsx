import React from 'react'

function Items() {
  return <>
    {items.map((item) => <Item key={item.key} />)}
  </>
}

const Items2 = () => <React.Fragment>hello</React.Fragment>
