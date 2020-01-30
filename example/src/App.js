import React from 'react';
import Grid from 'broker-grid';

const App = () => (
  <Grid endpoint={'http://localhost:8080'} eventListen={'user'} title={'Broker Grid'} token={'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxNGM3NjEwYS1lM2RhLTRkNzItOGYyYS1iYjJjZDYwYzNhOGEiLCJjb21wYW55IjoiIiwiZXhwIjoxNTgwNDA3Nzg0fQ.pLy_CToyb6KESsJMyAvlxX-FlmU74eRC1HXn7X5Lr1c'} />
);

export default App;
