import React from 'react';
import { makeStyles } from '@material-ui/core/styles';
import Card from '@material-ui/core/Card';
import { useSSE, SSEProvider } from 'react-hooks-sse';
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
        value: null,
      },
    },
    stateReducer(state, changes) {
      return changes;
    },
    parser(input) {
      return JSON.parse(input);
    },
  });

  return <p>{state.data.user !== null && <span>{state.data.user}</span>}</p>;
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
      headers: new Headers({
        authorization: 'Bearer 123',
      })
    });
    sse.addEventListener('user', (messageEvent) => {
      console.log(messageEvent);
    });
  };

  return (
    <div>
      <Card className={classes.card}>
       <CardContent>
        <Typography className={classes.title} color="textSecondary" gutterBottom component={'span'} variant={'body2'}>
          What is your name?&nbsp;
            <SSEProvider endpoint={sseEndpoint} options={{withCredentials: false}}>
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
