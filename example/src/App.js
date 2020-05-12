import React, {useEffect, useState} from 'react';
import Grid from 'broker-grid';
import {
  BrowserRouter as Router,
  Switch,
  Route
} from "react-router-dom";
import { useForm } from "react-hook-form";
import Logo from './logo.svg';
import uuid from 'uuid/v4';
import './tailwind.css';
import './spinner.css';

function Insert(props) {
  const { handleSubmit, register } = useForm();
  const stamp = Math.floor(Date.now() / 1000);
  const id = uuid();
  const onSubmit = values => {
    const apiEndpoint = process.env.REACT_APP_API + '/insert';
    const vals = JSON.stringify(values);
    const v = `{"collection_id": "${id}", "tenant_id":"112718d1-a0be-4468-b902-0749c3d964ae", "event": "covid", "timestamp": ${stamp}, "data": ${vals} }`;
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
    <form class="w-full max-w-sm mx-20" onSubmit={handleSubmit(onSubmit)}>
      <img src={Logo} alt="logo" class="mb-10 mt-5" />
      <div class="md:flex md:items-center mb-6">
        <div class="md:w-2/3">
          <input
            ref={register}
            name="Username"
            placeholder="Username"
            class="bg-gray-200 appearance-none border-2 border-gray-200 rounded w-full py-2 px-4 text-gray-700 leading-tight focus:outline-none focus:bg-white focus:border-purple-500"
          />
        </div>
      </div>
      <div class="md:flex md:items-center mb-6">
        <div class="md:w-2/3">
          <input
            ref={register}
            name="message"
            placeholder="Message"
            class="bg-gray-200 appearance-none border-2 border-gray-200 rounded w-full py-2 px-4 text-gray-700 leading-tight focus:outline-none focus:bg-white focus:border-purple-500"
          />
        </div>
      </div>
      <div class="md:flex md:items-center mb-6">
        <div class="md:w-2/3">
          <input
            ref={register}
            name="location"
            placeholder="Location"
            class="bg-gray-200 appearance-none border-2 border-gray-200 rounded w-full py-2 px-4 text-gray-700 leading-tight focus:outline-none focus:bg-white focus:border-purple-500"
          />
        </div>
      </div>
      <div class="md:flex md:items-center">
        <div class="md:w-1/3"></div>
        <div class="md:w-2/3">
          <button class="shadow bg-teal-500 hover:bg-teal-400 focus:shadow-outline focus:outline-none text-white font-bold py-2 px-4 rounded" type="submit">
            Submit
          </button>
        </div>
      </div>
    </form>
  );
}

function Load(props) {
  const { handleSubmit, register, errors } = useForm();
  const sseURL = `${process.env.REACT_APP_API}/events/${process.env.REACT_APP_TENANT}`;
  const insertURL = `${process.env.REACT_APP_API}/insert`;
  return (
    <Router>
      <div>
        <Switch>
          <Route path="/">
            <Insert jwt={props.jwt} />
              { props.jwt.length == 0 && 
                <div class="spinner mt-20">
                </div>
              }
              {props.jwt.length > 0 && 
                <div class="mt-20 mx-20">
                  <Grid sseEndpoint={sseURL} tenantID={process.env.REACT_APP_TENANT} insertEndpoint={insertURL} eventListen={'covid'} title={'Fight Covid'} token={props.jwt} />
                </div>
              }
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
