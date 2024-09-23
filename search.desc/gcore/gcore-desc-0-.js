searchState.loadedDescShard("gcore", 0, "Lightweight library for use in Gear programs.\nProgram (actor) identifier.\nRepresents block count type.\nRepresents block number type.\nCode identifier.\nCurrent version of execution settings.\nRepresents gas type.\nType representing converter between gas and value.\nMessage handle.\nMessage identifier.\nBasic struct for working with integer percentages allowing …\nReservation identifier.\nRepresents SS58 address.\nRepresents value type.\nReturns string slice containing SS58 address.\nType definitions and helpers for error handling.\nUtility functions related to the current execution context …\nCurrent value of existential deposit.\nExtensions for additional features.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCreates GasMultiplier from gas per value multiplier.\nCreates GasMultiplier from value per gas multiplier.\nCurrent gas multiplier.\nConverts given gas amount into its value equivalent.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns <code>ActorId</code>as bytes array.\nReturns <code>MessageId</code>as bytes array.\nReturns <code>CodeId</code>as bytes array.\nReturns <code>ReservationId</code>as bytes array.\nChecks whether <code>ActorId</code> is zero.\nChecks whether <code>MessageId</code> is zero.\nChecks whether <code>CodeId</code> is zero.\nChecks whether <code>ReservationId</code> is zero.\nCurrent value of mailbox threshold.\nMessaging API for Gear programs.\nCreates a new <code>ActorId</code> from a 32-byte array.\nCreates a new <code>MessageId</code> from a 32-byte array.\nCreates a new <code>CodeId</code> from a 32-byte array.\nCreates a new <code>ReservationId</code> from a 32-byte array.\nCreates GasMultiplier with gas == value.\nCurrent performance multiplier.\nAPI for creating programs from Gear programs.\nReturns <code>H160</code> with possible loss of the first 12 bytes.\nReturns the ss58-check address with default ss58 version.\nReturns the ss58-check address with given ss58 version.\nConverts given value amount into its gas equivalent, …\nCreates a new zero <code>ActorId</code>.\nCreates a new zero <code>MessageId</code>.\nCreates a new zero <code>CodeId</code>.\nCreates a new zero <code>ReservationId</code>.\nThe error occurs in attempt to access memory outside wasm …\nSuccess reply was created by system automatically.\nExecution failed with backend error that couldn’t been …\nGiven code id for program creation doesn’t exist.\nThe error type returned when conversion fails.\nThe error occurs in attempt to initialize the same program …\nThe error occurs in case of attempt to send more than one …\nThe error occurs when program tries to create reply …\nThe error occurs in attempt to get the same message from …\nContains the error value\nCommon error type returned by API functions from other …\nError reply.\nReason of error reply creation.\nError reply was created due to underlying execution error.\nSignal was sent due to some execution errors.\nExecution error.\nExecution error.\nAPI error (see <code>ExtError</code> for details).\nAn error occurred in API.\nError reply was created due to errors in program creation.\nDestination actor is inactive, so it can’t process the …\nAn error occurs in attempt to send or push reply while …\nThe error occurs when program tries to create reply …\nAn error occurs in attempt to charge gas for dispatch …\nEverything less than mailbox threshold but greater than 0 …\nEverything less than existential deposit but greater than …\nInvalid hex string.\nAn error occurs in attempt to unreserve gas with …\nInvalid slice length.\nInvalid SS58 address.\nAn attempt to commit or push a payload into an already …\nSuccess reply was created by actor manually.\nMessage has bigger then allowed one message size\nMemory error.\nMemory error.\nProgram has reached memory limit while executing.\nMessage error.\nError using messages.\nThe error occurs when functions related to reply context, …\nThe error occurs when functions related to signal context, …\nThe error occurs when functions related to status code, …\nAn error occurs in attempt to charge more gas than …\nThe error occurs when balance is less than required by …\nContains the success value\nThe error occurs in case of not valid identifier specified.\nThe error occurs when a too big length value to form a …\nThe error occurs when a wrong offset of the input buffer …\nThe error “Message limit exceeded” occurs when a …\nThe error occurs when program tries to send messages with …\nMessage ran out of gas while executing.\nCannot take data in payload range\nProgram re-instrumentation failed.\nMessage has died in Waitlist as out of rent one.\nSignal was sent due to removal from waitlist as out of …\nEnum representing reply code with reason of its creation.\nReservation error.\nAn error occurs in attempt to reserve gas less than …\nReservation error.\nAn error occurs in attempt to reserve more times than …\n<code>Result</code> type with a predefined error type (<code>ExtError</code>).\nThe error occurs, when program tries to allocate in …\nEnum representing signal code and reason of its creation.\nSimplified error occurred during execution.\nSimplified error occurred during program creation.\nSS58 encoding failed.\nProgram has reached stack limit while executing.\nSuccess reply.\nReason of success reply creation.\nSyscall executing result.\nSyscall usage error.\nOverflow in ‘gr_read’\nExecution failed with <code>unreachable</code> instruction call.\nUnsupported code. Variant exists for backward …\nUnsupported reason of success reply. Variant exists for …\nUnsupported reason of error reply. Variant exists for …\nUnsupported reason of execution error. Variant exists for …\nUnsupported reason of program creation error. Variant …\nThere is a new error variant old program don’t support.\nExecution failed with userspace panic.\nAn error occurs in attempt to reserve zero gas.\nAn error occurs in attempt to create reservation for 0 …\nConstructs <code>ReplyCode::Error(_)</code> variant from underlying …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nParses 4 bytes array to <code>ReplyCode</code>.\nParses <code>SignalCode</code> from <code>u32</code> if possible.\nConvert code into error.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConvert <code>SyscallError</code> into <code>Result</code>.\nReturns bool, defining if <code>ReplyCode</code> represents error reply.\nReturns bool, defining if <code>ReplyCode</code> represents success …\nReturns bool, defining if <code>ReplyCode</code> represents unsupported …\nConverts <code>ReplyCode</code> to 4 bytes array.\nConverts <code>SignalCode</code> into <code>u32</code>.\nConvert error into code.\nGet the current block height.\nGet the current block timestamp.\nGet current version of environment variables.\nTerminate the execution of a program.\nGet the current amount of gas available for execution.\nBreak the current execution.\nReturn the identifier of the current program.\nGet the random seed, along with the block number from …\nProvide gas deposit from current message to handle reply …\nReserve the <code>amount</code> of gas for further usage.\nReserve the <code>amount</code> of gas for system usage.\nUnreserve gas identified by <code>ReservationId</code>.\nGet the total available value amount.\nPause the current message handling.\nSame as <code>wait</code>, but delays handling for a specific number of …\nSame as <code>wait</code>, but delays handling for the maximum number …\nResume previously paused message handling.\nSame as <code>wake</code>, but executes after the <code>delay</code> expressed in …\nAdd a <code>data</code> string to the debug log.\nOut of memory panic\nPanic\nGet an identifier of the message that is currently being …\nGet a payload of the message that is currently being …\nGet a payload of the message that is currently being …\nSend a new message as a reply to the message that is …\nGet the reply code of the message being processed.\nFinalize and send the current reply message.\nSame as <code>reply_commit</code>, but it spends gas from a reservation …\nSame as <code>reply_commit</code>, but with an explicit gas limit.\nSame as <code>reply</code>, but it spends gas from a reservation …\nSame as <code>reply</code>, but relays the incoming message payload.\nSame as <code>reply_input</code>, but with explicit gas limit.\nPush a payload part to the current reply message.\nSame as <code>reply_push</code> but uses the input buffer as a payload …\nGet an identifier of the initial message on which the …\nSame as <code>reply</code>, but with an explicit gas limit.\nSend a new message to the program or user.\nFinalize and send the message formed in parts.\nSame as <code>send_commit</code>, but sends the message after the <code>delay</code> …\nSame as <code>send_commit_from_reservation</code>, but sends the …\nSame as <code>send_commit</code>, but it spends gas from a reservation …\nSame as <code>send_commit</code>, but with an explicit gas limit.\nSame as <code>send_commit_with_gas</code>, but sends the message after …\nSame as <code>send</code>, but sends the message after the <code>delay</code> …\nSame as <code>send_from_reservation</code>, but sends the message after …\nSame as <code>send</code>, but it spends gas from a reservation instead …\nInitialize a message to send formed in parts.\nSame as <code>send</code> but uses the input buffer as a payload source.\nSame as <code>send_input</code>, but sends delayed.\nSame as <code>send_input</code>, but with explicit gas limit.\nSame as <code>send_input_with_gas</code>, but sends delayed.\nPush a payload part of the message to be sent in parts.\nSame as <code>send_push</code> but uses the input buffer as a payload …\nSame as <code>send</code>, but with an explicit gas limit.\nSame as <code>send_with_gas</code>, but sends the message after the …\nGet the reply code of the message being processed.\nGet an identifier of the message which issued a signal.\nGet the payload size of the message that is being …\nGet the identifier of the message source (256-bit address).\nGet the value associated with the message that is being …\nExecutes function <code>f</code> with provided message payload …\nCreate a new program and returns its address.\nSame as <code>create_program</code>, but creates a new program after …\nSame as <code>create_program</code>, but with an explicit gas limit.\nSame as <code>create_program_with_gas</code>, but creates a new program …")