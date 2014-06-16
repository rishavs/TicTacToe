var express = require('express');
var path = require('path');
var favicon = require('static-favicon');
var logger = require('morgan');
var cookieParser = require('cookie-parser');
var bodyParser = require('body-parser');
var sockjs  = require('sockjs');

var routes = require('./routes/index');
var users = require('./routes/users');

var app = express();

 // view engine setup
app.set('views', path.join(__dirname, 'views'));
app.set('view engine', 'jade');

app.use(favicon());
app.use(logger('dev'));
app.use(bodyParser.json());
app.use(bodyParser.urlencoded());
app.use(cookieParser());
app.use(express.static(path.join(__dirname, 'public')));

app.use('/', routes);
app.use('/users', users);

/// catch 404 and forward to error handler
app.use(function(req, res, next) {
    var err = new Error('Not Found');
    err.status = 404;
    next(err);
});

// error handlers

// development error handler
// will print stacktrace
if (app.get('env') === 'development') {
    app.use(function(err, req, res, next) {
        res.status(err.status || 500);
        res.render('error', {
            message: err.message,
            error: err
        });
    });
}

// production error handler
// no stacktraces leaked to user
app.use(function(err, req, res, next) {
    res.status(err.status || 500);
    res.render('error', {
        message: err.message,
        error: {}
    });
});

var echo = sockjs.createServer();
var currentConnections = [];

echo.on('connection', function(conn) {
    console.log('\n[' + conn.id + ']' + ' JOINED');
	currentConnections.push(conn);
	
	//show a welcoming message only to this specific client
	conn.write('[SERVER WHISPER TO YOU] :' + 'Hello ' + '[' + conn.id + ']' + '. Welcome!');

    conn.on('data', function(message) {
        console.log('\n[' + conn.id + ']' + ' sent message : ' + message);
        broadcast('[SERVER BROADCAST TO ALL] : ' + '[' + conn.id + ']' + ' sent message : ' + '<b>' + message + '</b>');
    });
	
	conn.on('close', function() {
		console.log('\n[' + conn.id + ']' + ' LEFT');
		
		// remove the closed connection from the list
		ToolBelt.removeFromArray(currentConnections, conn);
    });
});

function broadcast(message){
  // iterate through each client in clients object
    // write the message to all connected clients
    for (var i=0; i<currentConnections.length; i++) {
      currentConnections[i].write(message);
	  console.log('Broadcasting to ' + currentConnections[i].id);
    }
}

function removeFromArray (array, element ) {
	// I seriously have to write this? wow... how bass-ackwards is this lang?
	while (array.indexOf(element) !== -1) {
		array.splice(array.indexOf(element), 1);
	}
}

var server = app.listen(3000, function() {
    console.log('Listening on port %d', server.address().port);
});

echo.installHandlers(server, {prefix:'/echo'});

module.exports = app;
