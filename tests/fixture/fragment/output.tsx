import React from 'react';
function Items() {
    return <>
    {items.map((item, __testidIdx0)=><Item data-testid={"Items.Item-" + __testidIdx0} key={item.key}/>)}
  </>;
}
const Items2 = ()=><React.Fragment data-testid="Items2.Fragment">hello</React.Fragment>;
