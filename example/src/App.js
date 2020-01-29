import React from 'react';
import { useSSE, SSEProvider } from 'broker-hook';
import Wrapper from './Wrapper';
import MaterialTable from "material-table";
import uuid from 'uuid/v4';

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

const eventListen = 'user';
const apiEndpoint = process.env.REACT_APP_API;
const token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxNGM3NjEwYS1lM2RhLTRkNzItOGYyYS1iYjJjZDYwYzNhOGEiLCJjb21wYW55IjoiIiwiZXhwIjoxNTgwMzM5MTkwfQ.rQTEPK3bovxjfmpRBnfIVS-Ki6qq53UVt8J4Qb5Vm5M";

const Comments = () => {
  const state = useSSE(eventListen, {
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
      return JSON.parse(input)
    },
  });

  return <div>{state.data.events != null && 
    <Wrapper>
    <MaterialTable
          icons={tableIcons}
          columns={state.data.columns}
          data={state.data.rows}
          title="Demo Title"
          editable={{
            onRowAdd: newData =>
              new Promise((resolve, reject) => {
                setTimeout(() => {
                  {
                    const id = uuid();
                    const ts = Math.round((new Date()).getTime() / 1000);
                    const v = `{"event": "${eventListen}", "collection_id": "${id}", "timestamp": ${ts}, "data": "${newData}"}`;
                    console.log(newData);
                    fetch(apiEndpoint, {
                      method: 'post',
                      mode: 'cors',
                      headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${token}`
                      },
                      body: v
                    }).then(response => {
                      return response.json();
                    }, err => {
                      console.log(err);
                    });
                  };
                  resolve()
                }, 1000)
              }),
            onRowUpdate: (newData, oldData) =>
              new Promise((resolve, reject) => {
                setTimeout(() => {
                  {
                    console.log('update');
                  }
                  resolve()
                }, 1000)
              }),
          }}
        /></Wrapper>}</div>;
};

function App() {
  const sseEndpoint = process.env.REACT_APP_EVENTS;

  return (
    <div>
      <SSEProvider endpoint={sseEndpoint} options={{headers: {authorization: `Bearer ${token}`}}}>
        <Comments />
      </SSEProvider>
    </div>
  );
}

export default App;
