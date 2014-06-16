var sock = new SockJS('/echo');

sock.onopen = function() {
	addText("You have JOINED the session");
};
sock.onmessage = function(e) {
	console.log('message', e.data);

	// Append the text to text area 
	addText(e.data);
};
sock.onclose = function() {
	addText("You have LEFT the session");
};

sock.send("Dude joined and stuff...");

function sendRock () {
	sock.send("Rock");
}

function sendPaper () {
	sock.send("Paper");
}

function sendScissors () {
	sock.send("Scissors");
}

function addText(text) {
	// Append the text to text area (using jQuery)
	$('#wallOfText').append('<p>' + text + '</p>');
	
	// scroll to the bottom of the div
	$('#wallOfText').scrollTop($('#wallOfText').prop('scrollHeight'));
}