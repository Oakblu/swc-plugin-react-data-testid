import React from 'react';
function Items() {
    return <>
    {items.map((item)=><Item data-testid="Items.Item" key={item.key}/>)}
  </>;
}
const Items2 = ()=><React.Fragment data-testid="Items2.Fragment">hello</React.Fragment>;
