module.exports = function ToolBelt() {
    removeFromArray : function (array, element ) {
        // I seriously have to write this? wow... how bass-ackwards is this lang?
        while (array.indexOf(element) !== -1) {
            array.splice(array.indexOf(element), 1);
        }
    }

    var server = app.listen(3000, function() {
        console.log('Listening on port %d', server.address().port);
    });

    echo.installHandlers(server, {prefix:'/echo'});

}