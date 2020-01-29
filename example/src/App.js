import React from 'react';
import { useSSE, SSEProvider } from 'broker-hook';
import Wrapper from './Wrapper';
import MaterialTable from "material-table";

import { forwardRef } from 'react';

import AddBox from '@material-ui/icons/AddBox';
import ArrowDownward from '@material-ui/icons/ArrowDownward';
import Check from '@material-ui/icons/Check';
import ChevronLeft from '@material-ui/icons/ChevronLeft';
import ChevronRight from '@material-ui/icons/ChevronRight';
import Clear from '@material-ui/icons/Clear';
import DeleteOutline from '@material-ui/icons/DeleteOutline';
import Edit from '@material-ui/icons/Edit';
import FilterList from '@material-ui/icons/FilterList';
import FirstPage from '@material-ui/icons/FirstPage';
import LastPage from '@material-ui/icons/LastPage';
import Remove from '@material-ui/icons/Remove';
import SaveAlt from '@material-ui/icons/SaveAlt';
import Search from '@material-ui/icons/Search';
import ViewColumn from '@material-ui/icons/ViewColumn';

const tableIcons = {
    Add: forwardRef((props, ref) => <AddBox {...props} ref={ref} />),
    Check: forwardRef((props, ref) => <Check {...props} ref={ref} />),
    Clear: forwardRef((props, ref) => <Clear {...props} ref={ref} />),
    Delete: forwardRef((props, ref) => <DeleteOutline {...props} ref={ref} />),
    DetailPanel: forwardRef((props, ref) => <ChevronRight {...props} ref={ref} />),
    Edit: forwardRef((props, ref) => <Edit {...props} ref={ref} />),
    Export: forwardRef((props, ref) => <SaveAlt {...props} ref={ref} />),
    Filter: forwardRef((props, ref) => <FilterList {...props} ref={ref} />),
    FirstPage: forwardRef((props, ref) => <FirstPage {...props} ref={ref} />),
    LastPage: forwardRef((props, ref) => <LastPage {...props} ref={ref} />),
    NextPage: forwardRef((props, ref) => <ChevronRight {...props} ref={ref} />),
    PreviousPage: forwardRef((props, ref) => <ChevronLeft {...props} ref={ref} />),
    ResetSearch: forwardRef((props, ref) => <Clear {...props} ref={ref} />),
    Search: forwardRef((props, ref) => <Search {...props} ref={ref} />),
    SortArrow: forwardRef((props, ref) => <ArrowDownward {...props} ref={ref} />),
    ThirdStateCheck: forwardRef((props, ref) => <Remove {...props} ref={ref} />),
    ViewColumn: forwardRef((props, ref) => <ViewColumn {...props} ref={ref} />)
};


const Comments = () => {
  const state = useSSE('user', {
    initialState: {
      data: {
        events: null,
        rows: null,
        columns: null
      },
    },
    stateReducer(state, changes) {
      return changes;
    },
    parser(input) {
      console.log(input)
      return JSON.parse(input)
    },
  });

  console.log(state.data);

  return <div>{state.data.events != null && 
    <Wrapper>
    <MaterialTable
          icons={tableIcons}
          columns={state.data.columns}
          data={state.data.rows}
          title="Demo Title"
        /></Wrapper>}</div>;
};

function App() {
  const sseEndpoint = process.env.REACT_APP_EVENTS;
  const apiEndpoint = process.env.REACT_APP_API;

  return (
    <div>
      <SSEProvider endpoint={sseEndpoint} options={{headers: {authorization: 'Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI3OGJjZDYxNC1jZDM5LTQzMWEtYWIyNC04OWQ5MTlkYmJmODkiLCJjb21wYW55IjoiIiwiZXhwIjoxNTgwMjU2ODA4fQ.cYFclXygM8AM_bt5I7lyGRZDhW_LL1Z1ZFgV5EHbnoI'}}}>
        <Comments />
      </SSEProvider>
    </div>
  );
}

export default App;
