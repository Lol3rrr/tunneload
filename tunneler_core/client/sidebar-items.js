initSidebarItems({"struct":[["Client","The Client instance in general that is responsible for handling all the interactions with the Server"],["QueueSender","Handles all the sending related to a single user-connection as well as the correct clean up handling once this is dropped"]],"trait":[["Handler","This defines a single Handler that will receive every new Connection that is established"],["Receiver","A Generic trait that abstracts over the actual receiving type and implementation allowing you to easily mock it and also allows for more versitility when using this in general as well as giving more freedom to the underlying types"],["Sender","A Generic trait that abstract away the actual underlying type and implementation for sending Data back to the User over that connection"]]});