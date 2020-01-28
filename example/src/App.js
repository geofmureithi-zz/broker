import React from 'react';
import { makeStyles } from '@material-ui/core/styles';
import Card from '@material-ui/core/Card';
import { useSSE, SSEProvider } from 'broker-hook';
import CardContent from '@material-ui/core/CardContent';
import Typography from '@material-ui/core/Typography';
import { useForm } from "react-hook-form";
import {DebounceInput} from 'react-debounce-input';
import BrokerClient from 'broker-client';

const useStyles = makeStyles({
  card: {
    minWidth: 275,
    position: "absolute",
    top: "50%",
    left: "50%", 
    backgroundColor: "yellow",
    transform: "translateX(-50%) translateY(-50%)"
  },
  title: {
    fontSize: 25,
  },
});

const Comments = () => {
  const state = useSSE('user', {
    initialState: {
      data: {
        events: null,
      },
    },
    stateReducer(state, changes) {
      return changes;
    },
    parser(input) {
      return JSON.parse(input)
    },
  });

  console.log(state.data);

  return <p>{state.data.events != null && <span>{state.data.events[0].data.user}</span>}</p>;
};

function App() {
  const classes = useStyles();
  const sseEndpoint = process.env.REACT_APP_EVENTS;
  const apiEndpoint = process.env.REACT_APP_API;
  const { handleSubmit, register, errors } = useForm();
  const onSubmit = values => {
    const ts = Math.round((new Date()).getTime() / 1000);
    const v = `{"event": "user", "published": false, "timestamp": ${ts}, "data": {"user": "${values.user}"}}`;
    const sse = new BrokerClient('http://localhost:8080/events', {
      headers: {
        authorization: 'Bearer 123',
      }
    });
    sse.addEventListener('internal_status', (messageEvent) => {
      console.log(messageEvent);
    });
  };

  return (
    <div>
      <Card className={classes.card}>
       <CardContent>
        <Typography className={classes.title} color="textSecondary" gutterBottom component={'span'} variant={'body2'}>
          What is your name?&nbsp;
            <SSEProvider endpoint={sseEndpoint} options={{headers: {authorization: 'Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI3OGJjZDYxNC1jZDM5LTQzMWEtYWIyNC04OWQ5MTlkYmJmODkiLCJjb21wYW55IjoiIiwiZXhwIjoxNTgwMjU2ODA4fQ.cYFclXygM8AM_bt5I7lyGRZDhW_LL1Z1ZFgV5EHbnoI'}}}>
              <Comments />
            </SSEProvider>
          </Typography>
          <form>
              <label htmlFor="user">Name: </label>
              <DebounceInput
                name="user"
                minLength={2}
                debounceTimeout={500}
                onChange={handleSubmit(onSubmit)}
                inputRef={register({})}
              />
              {errors.user && errors.user.message}
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

export default App;
