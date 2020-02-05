import React, {useEffect, useState} from 'react';
import Grid from 'broker-grid';
import {
  BrowserRouter as Router,
  Switch,
  Route
} from "react-router-dom";
import { useForm } from "react-hook-form";
import { Input, Button } from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';
import Card from '@material-ui/core/Card';
import CardContent from '@material-ui/core/CardContent';
import CardHeader from '@material-ui/core/CardHeader';
import Logo from './logo.svg';
import uuid from 'uuid/v4';

const useStyles = makeStyles({
  card: {
    minWidth: "275px",
    width: "100%"
  },
  title: {
    fontSize: 14,
  },
  pos: {
    marginBottom: 12,
  },
});

function Insert(props) {
  const classes = useStyles();
  const { handleSubmit, register } = useForm();
  const stamp = Math.floor(Date.now() / 1000);
  const id = uuid();
  const onSubmit = values => {
    const apiEndpoint = process.env.REACT_APP_API + '/insert';
    const vals = JSON.stringify(values);
    const v = `{"collection_id": "${id}", "event": "baby", "timestamp": ${stamp}, "data": ${vals} }`;
    fetch(apiEndpoint, {
      method: 'post',
      mode: 'cors',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${props.jwt}`
      },
      body: v
    }).then(response => {
      return response.json();
    }).catch(err => {
      console.log(err);
    });
  };
  return (
    <Card className={classes.card} raised={true} style={{marginTop: "50px"}}>
    <CardContent>
      <img src={Logo} alt="logo" />
      <CardHeader title="Client's Info" style={{marginTop: "25px"}} />
      <form onSubmit={handleSubmit(onSubmit)}>
        <span style={{marginLeft: "25px"}}>
          <Input
            inputRef={register}
            name="client_name"
            placeholder="Client's Full Name"
            required={true}
          />
        </span>
        <span style={{marginLeft: "25px"}}>
          <Input
            inputRef={register}
            name="client_phone_number"
            placeholder="Client's Phone #"
            required={true}
          />
        </span>
        <span style={{marginLeft: "25px"}}>
          <Input
            inputRef={register}
            name="client_email"
            placeholder="Client's Email"
            required={true}
          />
        </span>
        <div style={{marginTop: "50px"}}>
          <Button variant="contained" color="primary" type="submit">Submit</Button>
        </div>
      </form>
    </CardContent>
  </Card>);
}

function Load(props) {
  const classes = useStyles();
  const { handleSubmit, register, errors } = useForm();

  return (
    <Router>
      <div>
        <Switch>
          <Route path="/">
            <Insert jwt={props.jwt} />
            <Card className={classes.card} raised={true} style={{marginTop: "50px"}}>
              <CardContent>
                {props.jwt.length > 0 && 
                  <div style={{marginTop: "50px"}}>
                    <Grid endpoint={process.env.REACT_APP_API} eventListen={'baby'} title={'Client Info'} token={props.jwt} />
                  </div>
                }
            </CardContent>
          </Card>
          </Route>
        </Switch>
      </div>
    </Router>
  );
}

export default function App() {
  const [data, setData] = useState({jwt: ''});
  useEffect(() => {
    const apiEndpoint = process.env.REACT_APP_API + '/login';
    const v = `{"username": "${process.env.REACT_APP_USERNAME}" ,"password": "${process.env.REACT_APP_PASSWORD}" }`;
    let jwt = fetch(apiEndpoint, {
      method: 'post',
      mode: 'cors',
      headers: {
        'Content-Type': 'application/json'
      },
      body: v
    }).then(response => {
      return response.json();
    }).catch(err => {
      console.log(err);
    });
    jwt.then(res => {
      setData(res);
    })
  }, []);

  if (data === undefined) {
     return (
      <div>Loading...</div>
     );
  } else {
    return (
      <Load jwt={data.jwt} />
    );
  }
}
