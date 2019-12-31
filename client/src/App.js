import React from 'react';
import { makeStyles } from '@material-ui/core/styles';
import Card from '@material-ui/core/Card';
import { useSSE, SSEProvider } from 'react-hooks-sse';
import CardContent from '@material-ui/core/CardContent';
import Typography from '@material-ui/core/Typography';
import { useForm } from "react-hook-form";
import {DebounceInput} from 'react-debounce-input';

const useStyles = makeStyles({
  card: {
    minWidth: 275,
    position: "absolute",
    top: "50%",
    left: "50%",
    width: "500px",
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

  return <span>{state.data.user !== null && <span>{state.data.user}</span>}</span>;
};

function App() {
  const classes = useStyles();
  const sseEndpoint = process.env.REACT_APP_EVENTS;
  const apiEndpoint = process.env.REACT_APP_API;
  const { handleSubmit, register, errors } = useForm();
  const onSubmit = values => {
    const v = `{"event": "user", "data": {"user": "${values.user}"}}`;
    fetch(apiEndpoint, {
      method: 'post',
      mode: 'cors',
      headers: {
        'Content-Type': 'application/json'
      },
      body: v
    }).then(response => {
      return response.json();
    }, err => {
      console.log(err);
    });
  };
  const onBlur = evt => {
    evt.target.value = "";
  };

  return (
    <div>
      <Card className={classes.card}>
       <CardContent>
        <Typography className={classes.title} color="textSecondary" gutterBottom>
          What is your name?&nbsp;
            <SSEProvider endpoint={sseEndpoint} options={{withCredentials: false}}>
              <Comments />
            </SSEProvider>
          </Typography>
          <form onChange={handleSubmit(onSubmit)}>
              <label htmlFor="user">Name: </label>
              <DebounceInput
                name="user"
                minLength={2}
                debounceTimeout={300}
                onChange={handleSubmit(onSubmit)}
                onBlur={onBlur}
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
