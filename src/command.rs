use common::{Date, Nullable, WebElement, FrameId, LocatorStrategy};
use error::{WebDriverResult, WebDriverError, ErrorStatus};
use httpapi::{Route, WebDriverExtensionRoute, VoidWebDriverExtensionRoute};
use regex::Captures;
use rustc_serialize::json::{ToJson, Json};
use std::collections::BTreeMap;
use std::default::Default;


#[derive(PartialEq)]
pub enum WebDriverCommand<T: WebDriverExtensionCommand> {
    NewSession(NewSessionParameters),
    DeleteSession,
    Get(GetParameters),
    GetCurrentUrl,
    GoBack,
    GoForward,
    Refresh,
    GetTitle,
    GetPageSource,
    GetWindowHandle,
    GetWindowHandles,
    CloseWindow,
    GetWindowSize,
    SetWindowSize(WindowSizeParameters),
    GetWindowPosition,
    SetWindowPosition(WindowPositionParameters),
    MaximizeWindow,
//    FullscreenWindow // Not supported in marionette
    SwitchToWindow(SwitchToWindowParameters),
    SwitchToFrame(SwitchToFrameParameters),
    SwitchToParentFrame,
    FindElement(LocatorParameters),
    FindElements(LocatorParameters),
    FindElementElement(WebElement, LocatorParameters),
    FindElementElements(WebElement, LocatorParameters),
    GetActiveElement,
    IsDisplayed(WebElement),
    IsSelected(WebElement),
    GetElementAttribute(WebElement, String),
    GetElementProperty(WebElement, String),
    GetCSSValue(WebElement, String),
    GetElementText(WebElement),
    GetElementTagName(WebElement),
    GetElementRect(WebElement),
    IsEnabled(WebElement),
    ExecuteScript(JavascriptCommandParameters),
    ExecuteAsyncScript(JavascriptCommandParameters),
    GetCookies,
    GetCookie(String),
    AddCookie(AddCookieParameters),
    DeleteCookies,
    DeleteCookie(String),
    GetTimeouts,
    SetTimeouts(TimeoutsParameters),
    ElementClick(WebElement),
    ElementTap(WebElement),
    ElementClear(WebElement),
    ElementSendKeys(WebElement, SendKeysParameters),
    PerformActions(ActionsParameters),
    ReleaseActions,
    DismissAlert,
    AcceptAlert,
    GetAlertText,
    SendAlertText(SendKeysParameters),
    TakeScreenshot,
    TakeElementScreenshot(WebElement),
    Status,
    Extension(T)
}

pub trait WebDriverExtensionCommand : Clone + Send + PartialEq {
    fn parameters_json(&self) -> Option<Json>;
}

#[derive(Clone, PartialEq)]
pub struct VoidWebDriverExtensionCommand;

impl WebDriverExtensionCommand for VoidWebDriverExtensionCommand {
    fn parameters_json(&self) -> Option<Json> {
        panic!("No extensions implemented");
    }
}

#[derive(PartialEq)]
pub struct WebDriverMessage <U: WebDriverExtensionRoute=VoidWebDriverExtensionRoute> {
    pub session_id: Option<String>,
    pub command: WebDriverCommand<U::Command>,
}

impl <U: WebDriverExtensionRoute> WebDriverMessage<U> {
    pub fn new(session_id: Option<String>, command: WebDriverCommand<U::Command>) -> WebDriverMessage<U> {
        WebDriverMessage {
            session_id: session_id,
            command: command,
        }
    }

    pub fn from_http(match_type: Route<U>, params: &Captures, body: &str, requires_body: bool) -> WebDriverResult<WebDriverMessage<U>> {
        let session_id = WebDriverMessage::<U>::get_session_id(params);
        let body_data = if requires_body {
            debug!("Got request body {}", body);
            match Json::from_str(body) {
                Ok(x @ Json::Object(_)) => x,
                Ok(_) => return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                        "Body was not a json object")),
                Err(_) => return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                         format!("Failed to decode request body as json: {}", body)))
            }
        } else {
            Json::Null
        };
        let command = match match_type {
            Route::NewSession => {
                let parameters: NewSessionParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::NewSession(parameters)
            },
            Route::DeleteSession => WebDriverCommand::DeleteSession,
            Route::Get => {
                let parameters: GetParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::Get(parameters)
            },
            Route::GetCurrentUrl => WebDriverCommand::GetCurrentUrl,
            Route::GoBack => WebDriverCommand::GoBack,
            Route::GoForward => WebDriverCommand::GoForward,
            Route::Refresh => WebDriverCommand::Refresh,
            Route::GetTitle => WebDriverCommand::GetTitle,
            Route::GetPageSource => WebDriverCommand::GetPageSource,
            Route::GetWindowHandle => WebDriverCommand::GetWindowHandle,
            Route::GetWindowHandles => WebDriverCommand::GetWindowHandles,
            Route::CloseWindow => WebDriverCommand::CloseWindow,
            Route::GetTimeouts => WebDriverCommand::GetTimeouts,
            Route::SetTimeouts => {
                let parameters: TimeoutsParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SetTimeouts(parameters)
            },
            Route::GetWindowSize => WebDriverCommand::GetWindowSize,
            Route::SetWindowSize => {
                let parameters: WindowSizeParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SetWindowSize(parameters)
            },
            Route::GetWindowPosition => WebDriverCommand::GetWindowPosition,
            Route::SetWindowPosition => {
                let parameters: WindowPositionParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SetWindowPosition(parameters)
            },
            Route::MaximizeWindow => WebDriverCommand::MaximizeWindow,
            Route::SwitchToWindow => {
                let parameters: SwitchToWindowParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SwitchToWindow(parameters)
            }
            Route::SwitchToFrame => {
                let parameters: SwitchToFrameParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SwitchToFrame(parameters)
            },
            Route::SwitchToParentFrame => WebDriverCommand::SwitchToParentFrame,
            Route::FindElement => {
                let parameters: LocatorParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::FindElement(parameters)
            },
            Route::FindElements => {
                let parameters: LocatorParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::FindElements(parameters)
            },
            Route::FindElementElement => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                let parameters: LocatorParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::FindElementElement(element, parameters)
            },
            Route::FindElementElements => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                let parameters: LocatorParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::FindElementElements(element, parameters)
            },
            Route::GetActiveElement => WebDriverCommand::GetActiveElement,
            Route::IsDisplayed => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::IsDisplayed(element)
            },
            Route::IsSelected => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::IsSelected(element)
            },
            Route::GetElementAttribute => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                let attr = try_opt!(params.name("name"),
                                    ErrorStatus::InvalidArgument,
                                    "Missing name parameter").to_string();
                WebDriverCommand::GetElementAttribute(element, attr)
            },
            Route::GetElementProperty => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                let property = try_opt!(params.name("name"),
                                        ErrorStatus::InvalidArgument,
                                        "Missing name parameter").to_string();
                WebDriverCommand::GetElementProperty(element, property)
            },
            Route::GetCSSValue => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                let property = try_opt!(params.name("propertyName"),
                                        ErrorStatus::InvalidArgument,
                                        "Missing propertyName parameter").to_string();
                WebDriverCommand::GetCSSValue(element, property)
            },
            Route::GetElementText => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::GetElementText(element)
            },
            Route::GetElementTagName => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::GetElementTagName(element)
            },
            Route::GetElementRect => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::GetElementRect(element)
            },
            Route::IsEnabled => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::IsEnabled(element)
            },
            Route::ElementClick => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::ElementClick(element)
            },
            Route::ElementTap => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::ElementTap(element)
            },
            Route::ElementClear => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::ElementClear(element)
            },
            Route::ElementSendKeys => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                let parameters: SendKeysParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::ElementSendKeys(element, parameters)
            },
            Route::ExecuteScript => {
                let parameters: JavascriptCommandParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::ExecuteScript(parameters)
            },
            Route::ExecuteAsyncScript => {
                let parameters: JavascriptCommandParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::ExecuteAsyncScript(parameters)
            },
            Route::GetCookies => {
                WebDriverCommand::GetCookies
            },
            Route::GetCookie => {
                let name = try_opt!(params.name("name"),
                                    ErrorStatus::InvalidArgument,
                                    "Missing name parameter").to_string();
                WebDriverCommand::GetCookie(name)
            },
            Route::AddCookie => {
                let parameters: AddCookieParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::AddCookie(parameters)
            },
            Route::DeleteCookies => {
                WebDriverCommand::DeleteCookies
            },
            Route::DeleteCookie => {
                let name = try_opt!(params.name("name"),
                                    ErrorStatus::InvalidArgument,
                                    "Missing name parameter").to_string();
                WebDriverCommand::DeleteCookie(name)
            },
            Route::PerformActions => {
                let parameters: ActionsParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::PerformActions(parameters)
            },
            Route::ReleaseActions => {
                WebDriverCommand::ReleaseActions
            },
            Route::DismissAlert => {
                WebDriverCommand::DismissAlert
            },
            Route::AcceptAlert => {
                WebDriverCommand::AcceptAlert
            },
            Route::GetAlertText => {
                WebDriverCommand::GetAlertText
            },
            Route::SendAlertText => {
                let parameters: SendKeysParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SendAlertText(parameters)
            },
            Route::TakeScreenshot => WebDriverCommand::TakeScreenshot,
            Route::TakeElementScreenshot =>  {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.to_string());
                WebDriverCommand::TakeElementScreenshot(element)
            },
            Route::Status => WebDriverCommand::Status,
            Route::Extension(ref extension) => {
                try!(extension.command(params, &body_data))
            }
        };
        Ok(WebDriverMessage::new(session_id, command))
    }

    fn get_session_id(params: &Captures) -> Option<String> {
        params.name("sessionId").map(|x| x.to_string())
    }
}

impl <U:WebDriverExtensionRoute> ToJson for WebDriverMessage<U> {
    fn to_json(&self) -> Json {
        let parameters = match self.command {
            WebDriverCommand::AcceptAlert |
            WebDriverCommand::CloseWindow |
            WebDriverCommand::ReleaseActions |
            WebDriverCommand::DeleteCookie(_) |
            WebDriverCommand::DeleteCookies |
            WebDriverCommand::DeleteSession |
            WebDriverCommand::DismissAlert |
            WebDriverCommand::ElementClear(_) |
            WebDriverCommand::ElementClick(_) |
            WebDriverCommand::ElementTap(_) |
            WebDriverCommand::GetActiveElement |
            WebDriverCommand::GetAlertText |
            WebDriverCommand::GetCookie(_) |
            WebDriverCommand::GetCookies |
            WebDriverCommand::GetCSSValue(_, _) |
            WebDriverCommand::GetCurrentUrl |
            WebDriverCommand::GetElementAttribute(_, _) |
            WebDriverCommand::GetElementProperty(_, _) |
            WebDriverCommand::GetElementRect(_) |
            WebDriverCommand::GetElementTagName(_) |
            WebDriverCommand::GetElementText(_) |
            WebDriverCommand::GetPageSource |
            WebDriverCommand::GetTimeouts |
            WebDriverCommand::GetTitle |
            WebDriverCommand::GetWindowHandle |
            WebDriverCommand::GetWindowHandles |
            WebDriverCommand::GetWindowPosition |
            WebDriverCommand::GetWindowSize |
            WebDriverCommand::GoBack |
            WebDriverCommand::GoForward |
            WebDriverCommand::IsDisplayed(_) |
            WebDriverCommand::IsEnabled(_) |
            WebDriverCommand::IsSelected(_) |
            WebDriverCommand::MaximizeWindow |
            WebDriverCommand::NewSession(_) |
            WebDriverCommand::Refresh |
            WebDriverCommand::Status |
            WebDriverCommand::SwitchToParentFrame |
            WebDriverCommand::TakeElementScreenshot(_) |
            WebDriverCommand::TakeScreenshot => {
                None
            },

            WebDriverCommand::AddCookie(ref x) => Some(x.to_json()),
            WebDriverCommand::ElementSendKeys(_, ref x) => Some(x.to_json()),
            WebDriverCommand::ExecuteAsyncScript(ref x) |
            WebDriverCommand::ExecuteScript(ref x) => Some(x.to_json()),
            WebDriverCommand::FindElementElement(_, ref x) => Some(x.to_json()),
            WebDriverCommand::FindElementElements(_, ref x) => Some(x.to_json()),
            WebDriverCommand::FindElement(ref x) => Some(x.to_json()),
            WebDriverCommand::FindElements(ref x) => Some(x.to_json()),
            WebDriverCommand::Get(ref x) => Some(x.to_json()),
            WebDriverCommand::PerformActions(ref x) => Some(x.to_json()),
            WebDriverCommand::SendAlertText(ref x) => Some(x.to_json()),
            WebDriverCommand::SetTimeouts(ref x) => Some(x.to_json()),
            WebDriverCommand::SetWindowPosition(ref x) => Some(x.to_json()),
            WebDriverCommand::SetWindowSize(ref x) => Some(x.to_json()),
            WebDriverCommand::SwitchToFrame(ref x) => Some(x.to_json()),
            WebDriverCommand::SwitchToWindow(ref x) => Some(x.to_json()),
            WebDriverCommand::Extension(ref x) => x.parameters_json(),
        };

        let mut data = BTreeMap::new();
        if let Some(parameters) = parameters {
            data.insert("parameters".to_string(), parameters);
        }
        Json::Object(data)
    }
}

pub trait Parameters: Sized {
    fn from_json(body: &Json) -> WebDriverResult<Self>;
}

#[derive(PartialEq)]
pub struct NewSessionParameters {
    pub desired: BTreeMap<String, Json>,
    pub required: BTreeMap<String, Json>,
}

impl NewSessionParameters {
    pub fn get(&self, name: &str) -> Option<&Json> {
        self.required.get(name).or_else(|| self.desired.get(name))
    }

    pub fn consume(&mut self, name: &str) -> Option<Json> {
        let required = self.required.remove(name);
        let desired = self.desired.remove(name);
        if required.is_some() {
            required
        } else {
            desired
        }
    }
}

impl Parameters for NewSessionParameters {
    fn from_json(body: &Json) -> WebDriverResult<NewSessionParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::UnknownError,
                            "Message body was not an object");

        let desired_capabilities =
            if let Some(capabilities) = data.get("desiredCapabilities") {
                try_opt!(capabilities.as_object(),
                         ErrorStatus::InvalidArgument,
                         "'desiredCapabilities' parameter is not an object").clone()
            } else {
                BTreeMap::new()
            };

        let required_capabilities =
            if let Some(capabilities) = data.get("requiredCapabilities") {
                try_opt!(capabilities.as_object(),
                         ErrorStatus::InvalidArgument,
                         "'requiredCapabilities' parameter is not an object").clone()
            } else {
                BTreeMap::new()
            };

        Ok(NewSessionParameters {
            desired: desired_capabilities,
            required: required_capabilities
        })
    }
}

impl ToJson for NewSessionParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("desiredCapabilities".to_owned(), self.desired.to_json());
        data.insert("requiredCapabilities".to_owned(), self.required.to_json());
        Json::Object(data)
    }
}


#[derive(PartialEq)]
pub struct GetParameters {
    pub url: String
}

impl Parameters for GetParameters {
    fn from_json(body: &Json) -> WebDriverResult<GetParameters> {
        let data = try_opt!(body.as_object(), ErrorStatus::UnknownError,
                            "Message body was not an object");
        let url = try_opt!(
            try_opt!(data.get("url"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'url' parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "'url' not a string");
        Ok(GetParameters {
            url: url.to_string()
        })
    }
}

impl ToJson for GetParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("url".to_string(), self.url.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct TimeoutsParameters {
    pub type_: String,
    pub ms: f64
}

impl Parameters for TimeoutsParameters {
    fn from_json(body: &Json) -> WebDriverResult<TimeoutsParameters> {
        let data = try_opt!(body.as_object(), ErrorStatus::UnknownError,
                            "Message body was not an object");
        let type_ = try_opt!(
            try_opt!(data.get("type"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'type' parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "'type' not a string");

        let ms = try_opt!(
            try_opt!(data.get("ms"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'ms' parameter").as_f64(),
            ErrorStatus::InvalidArgument,
            "'ms' not a float");
        return Ok(TimeoutsParameters {
            type_: type_.to_string(),
            ms: ms
        })
    }
}

impl ToJson for TimeoutsParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("type".to_string(), self.type_.to_json());
        data.insert("ms".to_string(), self.ms.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct WindowSizeParameters {
    pub width: u64,
    pub height: u64,
}

impl Parameters for WindowSizeParameters {
    fn from_json(body: &Json) -> WebDriverResult<WindowSizeParameters> {
        let data = try_opt!(body.as_object(), ErrorStatus::UnknownError,
                            "Message body was not an object");
        let height = try_opt!(
            try_opt!(data.get("height"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'height' parameter").as_u64(),
            ErrorStatus::InvalidArgument,
            "'height' is not a positive integer");
        let width = try_opt!(
            try_opt!(data.get("width"),
                     ErrorStatus::InvalidArgument,
                     "Missing width parameter").as_u64(),
            ErrorStatus::InvalidArgument,
            "'width' is not a positive integer");
        return Ok(WindowSizeParameters {
            height: height,
            width: width
        })
    }
}

impl ToJson for WindowSizeParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("width".to_string(), self.width.to_json());
        data.insert("height".to_string(), self.height.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct WindowPositionParameters {
    pub x: u64,
    pub y: u64,
}

impl Parameters for WindowPositionParameters {
    fn from_json(body: &Json) -> WebDriverResult<WindowPositionParameters> {
        let data = try_opt!(body.as_object(),
            ErrorStatus::UnknownError, "Message body was not an object");
        let xv = try_opt!(data.get("x"),
            ErrorStatus::InvalidArgument, "Missing 'x' parameters");
        let x = try_opt!(xv.as_u64(),
            ErrorStatus::InvalidArgument, "'x' is not a positive integer");
        let yv = try_opt!(data.get("y"),
            ErrorStatus::InvalidArgument, "Missing 'y' parameters");
        let y = try_opt!(yv.as_u64(),
            ErrorStatus::InvalidArgument, "'y' is not a positive integer");
        return Ok(WindowPositionParameters { x: x, y: y });
    }
}


impl ToJson for WindowPositionParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("x".to_string(), self.x.to_json());
        data.insert("y".to_string(), self.y.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct SwitchToWindowParameters {
    pub handle: String
}

impl Parameters for SwitchToWindowParameters {
    fn from_json(body: &Json) -> WebDriverResult<SwitchToWindowParameters> {
        let data = try_opt!(body.as_object(), ErrorStatus::UnknownError,
                            "Message body was not an object");
        let handle = try_opt!(
            try_opt!(data.get("handle"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'handle' parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "'handle' not a string");
        return Ok(SwitchToWindowParameters {
            handle: handle.to_string()
        })
    }
}

impl ToJson for SwitchToWindowParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("handle".to_string(), self.handle.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct LocatorParameters {
    pub using: LocatorStrategy,
    pub value: String
}

impl Parameters for LocatorParameters {
    fn from_json(body: &Json) -> WebDriverResult<LocatorParameters> {
        let data = try_opt!(body.as_object(), ErrorStatus::UnknownError,
                            "Message body was not an object");

        let using = try!(LocatorStrategy::from_json(
            try_opt!(data.get("using"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'using' parameter")));

        let value = try_opt!(
            try_opt!(data.get("value"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'value' parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "Could not convert using to string").to_string();

        return Ok(LocatorParameters {
            using: using,
            value: value
        })
    }
}

impl ToJson for LocatorParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("using".to_string(), self.using.to_json());
        data.insert("value".to_string(), self.value.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct SwitchToFrameParameters {
    pub id: FrameId
}

impl Parameters for SwitchToFrameParameters {
    fn from_json(body: &Json) -> WebDriverResult<SwitchToFrameParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::UnknownError,
                            "Message body was not an object");
        let id = try!(FrameId::from_json(try_opt!(data.get("id"),
                                                  ErrorStatus::UnknownError,
                                                  "Missing 'id' parameter")));

        Ok(SwitchToFrameParameters {
            id: id
        })
    }
}

impl ToJson for SwitchToFrameParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("id".to_string(), self.id.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct SendKeysParameters {
    pub value: Vec<char>
}

impl Parameters for SendKeysParameters {
    fn from_json(body: &Json) -> WebDriverResult<SendKeysParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Message body was not an object");
        let value_json = try_opt!(try_opt!(data.get("value"),
                                           ErrorStatus::InvalidArgument,
                                           "Missing 'value' parameter").as_array(),
                                  ErrorStatus::InvalidArgument,
                                  "Could not convert 'value' to array");

        let value = try!(value_json.iter().map(|x| {
           let str_value = try_opt!(x.as_string(),
                                    ErrorStatus::InvalidArgument,
                                    "Value was not a string");
            let chars = str_value.chars().collect::<Vec<char>>();
            if chars.len() != 1 {
                return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                               "Value was not a string"));
            }
            Ok(chars[0])
        }).collect::<Result<Vec<_>, _>>());

        Ok(SendKeysParameters {
            value: value
        })
    }
}

impl ToJson for SendKeysParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        let value_string: Vec<String> = self.value.iter().map(|x| {
            x.to_string()
        }).collect();
        data.insert("value".to_string(), value_string.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct JavascriptCommandParameters {
    pub script: String,
    pub args: Nullable<Vec<Json>>
}

impl Parameters for JavascriptCommandParameters {
    fn from_json(body: &Json) -> WebDriverResult<JavascriptCommandParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Message body was not an object");

        let args_json = try_opt!(data.get("args"),
                                 ErrorStatus::InvalidArgument,
                                 "Missing args parameter");

        let args = try!(Nullable::from_json(
            args_json,
            |x| {
                Ok((try_opt!(x.as_array(),
                             ErrorStatus::InvalidArgument,
                             "Failed to convert args to Array")).clone())
            }));

         //TODO: Look for WebElements in args?
        let script = try_opt!(
            try_opt!(data.get("script"),
                     ErrorStatus::InvalidArgument,
                     "Missing script parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "Failed to convert script to String");
        Ok(JavascriptCommandParameters {
            script: script.to_string(),
            args: args.clone()
        })
    }
}

impl ToJson for JavascriptCommandParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        //TODO: Wrap script so that it becomes marionette-compatible
        data.insert("script".to_string(), self.script.to_json());
        data.insert("args".to_string(), self.args.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct GetCookieParameters {
    pub name: Nullable<String>
}

impl Parameters for GetCookieParameters {
    fn from_json(body: &Json) -> WebDriverResult<GetCookieParameters> {
        let data = try_opt!(body.as_object(), ErrorStatus::InvalidArgument,
                            "Message body was not an object");
        let name_json = try_opt!(data.get("name"),
                                 ErrorStatus::InvalidArgument,
                                 "Missing 'name' parameter");
        let name = try!(Nullable::from_json(
            name_json,
            |x| {
                Ok(try_opt!(x.as_string(),
                            ErrorStatus::InvalidArgument,
                            "Failed to convert name to String").to_string())
            }));
        return Ok(GetCookieParameters {
            name: name
        })
    }
}

impl ToJson for GetCookieParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), self.name.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct AddCookieParameters {
    pub name: String,
    pub value: String,
    pub path: Nullable<String>,
    pub domain: Nullable<String>,
    pub expiry: Nullable<Date>,
    pub secure: bool,
    pub httpOnly: bool
}

impl Parameters for AddCookieParameters {
    fn from_json(body: &Json) -> WebDriverResult<AddCookieParameters> {
        if !body.is_object() {
            return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                           "Message body was not an object"));
        }

        let data = try_opt!(body.find("cookie").and_then(|x| x.as_object()),
                            ErrorStatus::UnableToSetCookie,
                            "Cookie parameter not found or not an object");

        let name = try_opt!(
            try_opt!(data.get("name"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'name' parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "'name' is not a string").to_string();

        let value = try_opt!(
            try_opt!(data.get("value"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'value' parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "'value' is not a string").to_string();

        let path = match data.get("path") {
            Some(path_json) => {
                try!(Nullable::from_json(
                    path_json,
                    |x| {
                        Ok(try_opt!(x.as_string(),
                                    ErrorStatus::InvalidArgument,
                                    "Failed to convert path to String").to_string())
                    }))
            },
            None => Nullable::Null
        };

        let domain = match data.get("domain") {
            Some(domain_json) => {
                try!(Nullable::from_json(
                    domain_json,
                    |x| {
                        Ok(try_opt!(x.as_string(),
                                    ErrorStatus::InvalidArgument,
                                    "Failed to convert domain to String").to_string())
                    }))
            },
            None => Nullable::Null
        };

        let expiry = match data.get("expiry") {
            Some(expiry_json) => {
                try!(Nullable::from_json(
                    expiry_json,
                    |x| {
                        Ok(Date::new(try_opt!(x.as_u64(),
                                              ErrorStatus::InvalidArgument,
                                              "Failed to convert expiry to Date")))
                    }))
            },
            None => Nullable::Null
        };

        let secure = match data.get("secure") {
            Some(x) => try_opt!(x.as_boolean(),
                                ErrorStatus::InvalidArgument,
                                "Failed to convert secure to boolean"),
            None => false
        };

        let http_only = match data.get("httpOnly") {
            Some(x) => try_opt!(x.as_boolean(),
                                ErrorStatus::InvalidArgument,
                                "Failed to convert httpOnly to boolean"),
            None => false
        };

        return Ok(AddCookieParameters {
            name: name,
            value: value,
            path: path,
            domain: domain,
            expiry: expiry,
            secure: secure,
            httpOnly: http_only
        })
    }
}

impl ToJson for AddCookieParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), self.name.to_json());
        data.insert("value".to_string(), self.value.to_json());
        data.insert("path".to_string(), self.path.to_json());
        data.insert("domain".to_string(), self.domain.to_json());
        data.insert("expiry".to_string(), self.expiry.to_json());
        data.insert("secure".to_string(), self.secure.to_json());
        data.insert("httpOnly".to_string(), self.httpOnly.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct TakeScreenshotParameters {
    pub element: Nullable<WebElement>
}

impl Parameters for TakeScreenshotParameters {
    fn from_json(body: &Json) -> WebDriverResult<TakeScreenshotParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Message body was not an object");
        let element = match data.get("element") {
            Some(element_json) => try!(Nullable::from_json(
                element_json,
                |x| {
                    Ok(try!(WebElement::from_json(x)))
                })),
            None => Nullable::Null
        };

        return Ok(TakeScreenshotParameters {
            element: element
        })
    }
}

impl ToJson for TakeScreenshotParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("element".to_string(), self.element.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct ActionsParameters {
    pub actions: Vec<ActionSequence>
}

impl Parameters for ActionsParameters {
    fn from_json(body: &Json) -> WebDriverResult<ActionsParameters> {
        try_opt!(body.as_object(),
                 ErrorStatus::InvalidArgument,
                 "Message body was not an object");
        let actions = try_opt!(
            try_opt!(body.find("actions"),
                     ErrorStatus::InvalidArgument,
                     "No actions parameter found").as_array(),
            ErrorStatus::InvalidArgument,
            "Parameter 'actions' was not an array");

        let mut result = Vec::with_capacity(actions.len());
        for chain in actions.iter() {
            result.push(try!(ActionSequence::from_json(chain)));
        }
        Ok(ActionsParameters {
            actions: result
        })
    }
}

impl ToJson for ActionsParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("actions".to_owned(),
                    self.actions.iter().map(|x| x.to_json()).collect::<Vec<Json>>().to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct ActionSequence {
    pub id: Nullable<String>,
    pub actions: ActionsType
}

impl Parameters for ActionSequence {
    fn from_json(body: &Json) -> WebDriverResult<ActionSequence> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Actions chain was not an object");

        let type_name = try_opt!(try_opt!(data.get("type"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing type parameter").as_string(),
                                 ErrorStatus::InvalidArgument,
                                 "Parameter ;type' was not a string");

        let id = match data.get("id") {
            Some(x) => Some(try_opt!(x.as_string(),
                                     ErrorStatus::InvalidArgument,
                                     "Parameter 'id' was not a string").to_owned()),
            None => None
        };


        // Note that unlike the spec we get the pointer parameters in ActionsType::from_json

        let actions = match type_name {
            "none" | "key" | "pointer" => try!(ActionsType::from_json(&body)),
            _ => return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                "Invalid action type"))
        };

        Ok(ActionSequence {
            id: id.into(),
            actions: actions
        })
    }
}

impl ToJson for ActionSequence {
    fn to_json(&self) -> Json {
        let mut data: BTreeMap<String, Json> = BTreeMap::new();
        data.insert("id".into(), self.id.to_json());
        let (action_type, actions) = match self.actions {
            ActionsType::Null(ref actions) => {
                ("none",
                 actions.iter().map(|x| x.to_json()).collect::<Vec<Json>>())
            }
            ActionsType::Key(ref actions) => {
                ("key",
                 actions.iter().map(|x| x.to_json()).collect::<Vec<Json>>())
            }
            ActionsType::Pointer(ref parameters, ref actions) => {
                data.insert("parameters".into(), parameters.to_json());
                ("pointer",
                 actions.iter().map(|x| x.to_json()).collect::<Vec<Json>>())
            }
        };
        data.insert("type".into(), action_type.to_json());
        data.insert("actions".into(), actions.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub enum ActionsType {
    Null(Vec<NullActionItem>),
    Key(Vec<KeyActionItem>),
    Pointer(PointerActionParameters, Vec<PointerActionItem>)
}

impl Parameters for ActionsType {
    fn from_json(body: &Json) -> WebDriverResult<ActionsType> {
        // These unwraps are OK as long as this is only called from ActionSequence::from_json
        let data = body.as_object().expect("Body should be a JSON Object");
        let actions_type = body.find("type").and_then(|x| x.as_string()).expect("Type should be a string");
        let actions_chain = try_opt!(try_opt!(data.get("actions"),
                                              ErrorStatus::InvalidArgument,
                                              "Missing actions parameter").as_array(),
                                     ErrorStatus::InvalidArgument,
                                     "Parameter 'actions' was not an array");
        match actions_type {
            "none" => {
                let mut actions = Vec::with_capacity(actions_chain.len());
                for action_body in actions_chain.iter() {
                    actions.push(try!(NullActionItem::from_json(action_body)));
                };
                Ok(ActionsType::Null(actions))
            },
            "key" => {
                let mut actions = Vec::with_capacity(actions_chain.len());
                for action_body in actions_chain.iter() {
                    actions.push(try!(KeyActionItem::from_json(action_body)));
                };
                Ok(ActionsType::Key(actions))
            },
            "pointer" => {
                let mut actions = Vec::with_capacity(actions_chain.len());
                let parameters = match data.get("parameters") {
                    Some(x) => try!(PointerActionParameters::from_json(x)),
                    None => Default::default()
                };

                for action_body in actions_chain.iter() {
                    actions.push(try!(PointerActionItem::from_json(action_body)));
                }
                Ok(ActionsType::Pointer(parameters, actions))
            }
            _ => panic!("Got unexpected action type after checking type")
        }
    }
}

#[derive(PartialEq)]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
}

impl Parameters for PointerType {
    fn from_json(body: &Json) -> WebDriverResult<PointerType> {
        match body.as_string() {
            Some("mouse") => Ok(PointerType::Mouse),
            Some("pen") => Ok(PointerType::Pen),
            Some("touch") => Ok(PointerType::Touch),
            Some(_) => Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "Unsupported pointer type"
            )),
            None => Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "Pointer type was not a string"
            ))
        }
    }
}

impl ToJson for PointerType {
    fn to_json(&self) -> Json {
        match *self {
            PointerType::Mouse => "mouse".to_json(),
            PointerType::Pen => "pen".to_json(),
            PointerType::Touch => "touch".to_json(),
        }.to_json()
    }
}

impl Default for PointerType {
    fn default() -> PointerType {
        PointerType::Mouse
    }
}

#[derive(Default, PartialEq)]
pub struct PointerActionParameters {
    pub pointer_type: PointerType
}

impl Parameters for PointerActionParameters {
    fn from_json(body: &Json) -> WebDriverResult<PointerActionParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Parameter 'parameters' was not an object");
        let pointer_type = match data.get("pointerType") {
            Some(x) => try!(PointerType::from_json(x)),
            None => PointerType::default()
        };
        Ok(PointerActionParameters {
            pointer_type: pointer_type
        })
    }
}

impl ToJson for PointerActionParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("pointerType".to_owned(),
                    self.pointer_type.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub enum NullActionItem {
    General(GeneralAction)
}

impl Parameters for NullActionItem {
    fn from_json(body: &Json) -> WebDriverResult<NullActionItem> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Actions chain was not an object");
        let type_name = try_opt!(
            try_opt!(data.get("type"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'type' parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "Parameter 'type' was not a string");
        match type_name {
            "pause" => Ok(NullActionItem::General(
                try!(GeneralAction::from_json(body)))),
            _ => return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                "Invalid type attribute"))
        }
    }
}

impl ToJson for NullActionItem {
    fn to_json(&self) -> Json {
        match self {
            &NullActionItem::General(ref x) => x.to_json(),
        }
    }
}

#[derive(PartialEq)]
pub enum KeyActionItem {
    General(GeneralAction),
    Key(KeyAction)
}

impl Parameters for KeyActionItem {
    fn from_json(body: &Json) -> WebDriverResult<KeyActionItem> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Key action item was not an object");
        let type_name = try_opt!(
            try_opt!(data.get("type"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'type' parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "Parameter 'type' was not a string");
        match type_name {
            "pause" => Ok(KeyActionItem::General(
                try!(GeneralAction::from_json(body)))),
            _ => Ok(KeyActionItem::Key(
                try!(KeyAction::from_json(body))))
        }
    }
}

impl ToJson for KeyActionItem {
    fn to_json(&self) -> Json {
        match *self {
            KeyActionItem::General(ref x) => x.to_json(),
            KeyActionItem::Key(ref x) => x.to_json()
        }
    }
}

#[derive(PartialEq)]
pub enum PointerActionItem {
    General(GeneralAction),
    Pointer(PointerAction)
}

impl Parameters for PointerActionItem {
    fn from_json(body: &Json) -> WebDriverResult<PointerActionItem> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Pointer action item was not an object");
        let type_name = try_opt!(
            try_opt!(data.get("type"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'type' parameter").as_string(),
            ErrorStatus::InvalidArgument,
            "Parameter 'type' was not a string");

        match type_name {
            "pause" => Ok(PointerActionItem::General(try!(GeneralAction::from_json(body)))),
            _ => Ok(PointerActionItem::Pointer(try!(PointerAction::from_json(body))))
        }
    }
}

impl ToJson for PointerActionItem {
    fn to_json(&self) -> Json {
        match self {
            &PointerActionItem::General(ref x) => x.to_json(),
            &PointerActionItem::Pointer(ref x) => x.to_json()
        }
    }
}

#[derive(PartialEq)]
pub enum GeneralAction {
    Pause(PauseAction)
}

impl Parameters for GeneralAction {
    fn from_json(body: &Json) -> WebDriverResult<GeneralAction> {
        match body.find("type").and_then(|x| x.as_string()) {
            Some("pause") => Ok(GeneralAction::Pause(try!(PauseAction::from_json(body)))),
            _ => Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                         "Invalid or missing type attribute"))
        }
    }
}

impl ToJson for GeneralAction {
    fn to_json(&self) -> Json {
        match self {
            &GeneralAction::Pause(ref x) => x.to_json()
        }
    }
}

#[derive(PartialEq)]
pub struct PauseAction {
    pub duration: u64
}

impl Parameters for PauseAction {
    fn from_json(body: &Json) -> WebDriverResult<PauseAction> {
        let default = Json::U64(0);
        Ok(PauseAction {
            duration: try_opt!(body.find("duration").unwrap_or(&default).as_u64(),
                               ErrorStatus::InvalidArgument,
                               "Parameter 'duration' was not a positive integer")
        })
    }
}

impl ToJson for PauseAction {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("type".to_owned(),
                    "pause".to_json());
        data.insert("duration".to_owned(),
                    self.duration.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub enum KeyAction {
    Up(KeyUpAction),
    Down(KeyDownAction)
}

impl Parameters for KeyAction {
    fn from_json(body: &Json) -> WebDriverResult<KeyAction> {
        match body.find("type").and_then(|x| x.as_string()) {
            Some("keyDown") => Ok(KeyAction::Down(try!(KeyDownAction::from_json(body)))),
            Some("keyUp") => Ok(KeyAction::Up(try!(KeyUpAction::from_json(body)))),
            Some(_) | None => Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                      "Invalid type attribute value for key action"))
        }
    }
}

impl ToJson for KeyAction {
    fn to_json(&self) -> Json {
        match self {
            &KeyAction::Down(ref x) => x.to_json(),
            &KeyAction::Up(ref x) => x.to_json(),
        }
    }
}

#[derive(PartialEq)]
pub struct KeyUpAction {
    pub value: char
}

impl Parameters for KeyUpAction {
    fn from_json(body: &Json) -> WebDriverResult<KeyUpAction> {
        let value_str = try_opt!(
                try_opt!(body.find("value"),
                         ErrorStatus::InvalidArgument,
                         "Missing value parameter").as_string(),
                ErrorStatus::InvalidArgument,
            "Parameter 'value' was not a string");
        if value_str.len() != 1 {
            return Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "Key code was not a single char"))
        }

        let value = value_str.chars().next().unwrap();
        Ok(KeyUpAction {
            value: value
        })
    }
}

impl ToJson for KeyUpAction {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("type".to_owned(),
                    "keyUp".to_json());
        data.insert("value".to_string(),
                    self.value.to_string().to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct KeyDownAction {
    pub value: char
}

impl Parameters for KeyDownAction {
    fn from_json(body: &Json) -> WebDriverResult<KeyDownAction> {
        let value_str = try_opt!(
                try_opt!(body.find("value"),
                         ErrorStatus::InvalidArgument,
                         "Missing value parameter").as_string(),
                ErrorStatus::InvalidArgument,
            "Parameter 'value' was not a string");
        if value_str.len() != 1 {
            return Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "Key code was not a single char"))
        }

        let value = value_str.chars().next().unwrap();
        Ok(KeyDownAction {
            value: value
        })
    }
}

impl ToJson for KeyDownAction {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("type".to_owned(),
                    "keyUp".to_json());
        data.insert("value".to_owned(),
                    self.value.to_string().to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub enum PointerAction {
    Up(PointerUpAction),
    Down(PointerDownAction),
    Move(PointerMoveAction),
    Cancel
}

impl Parameters for PointerAction {
    fn from_json(body: &Json) -> WebDriverResult<PointerAction> {
        match body.find("type").and_then(|x| x.as_string()) {
            Some("pointerUp") => Ok(PointerAction::Up(try!(PointerUpAction::from_json(body)))),
            Some("pointerDown") => Ok(PointerAction::Down(try!(PointerDownAction::from_json(body)))),
            Some("move") => Ok(PointerAction::Move(try!(PointerMoveAction::from_json(body)))),
            Some("cancel") => Ok(PointerAction::Cancel),
            Some(_) | None => Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "Missing or invalid type argument for pointer action"))
        }
    }
}

impl ToJson for PointerAction {
    fn to_json(&self) -> Json {
        match self {
            &PointerAction::Down(ref x) => x.to_json(),
            &PointerAction::Up(ref x) => x.to_json(),
            &PointerAction::Move(ref x) => x.to_json(),
            &PointerAction::Cancel => {
                let mut data = BTreeMap::new();
                data.insert("type".to_owned(),
                            "cancel".to_json());
                Json::Object(data)
            }
        }
    }
}

#[derive(PartialEq)]
pub struct PointerUpAction {
    pub button: u64,
}

impl Parameters for PointerUpAction {
    fn from_json(body: &Json) -> WebDriverResult<PointerUpAction> {
        let button = try_opt!(
            try_opt!(body.find("button"),
                     ErrorStatus::InvalidArgument,
                     "Missing button parameter").as_u64(),
            ErrorStatus::InvalidArgument,
            "Parameter 'button' was not a positive integer");

        Ok(PointerUpAction {
            button: button
        })
    }
}

impl ToJson for PointerUpAction {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("type".to_owned(),
                    "pointerUp".to_json());
        data.insert("button".to_owned(), self.button.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct PointerDownAction {
    pub button: u64,
}

impl Parameters for PointerDownAction {
    fn from_json(body: &Json) -> WebDriverResult<PointerDownAction> {
        let button = try_opt!(
            try_opt!(body.find("button"),
                     ErrorStatus::InvalidArgument,
                     "Missing button parameter").as_u64(),
            ErrorStatus::InvalidArgument,
            "Parameter 'button' was not a positive integer");

        Ok(PointerDownAction {
            button: button
        })
    }
}

impl ToJson for PointerDownAction {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("type".to_owned(),
                    "pointerDown".to_json());
        data.insert("button".to_owned(), self.button.to_json());
        Json::Object(data)
    }
}

#[derive(PartialEq)]
pub struct PointerMoveAction {
    pub duration: Nullable<u64>,
    pub element: Nullable<WebElement>,
    pub x: Nullable<u64>,
    pub y: Nullable<u64>
}

impl Parameters for PointerMoveAction {
    fn from_json(body: &Json) -> WebDriverResult<PointerMoveAction> {
        let duration = match body.find("duration") {
            Some(duration) => Some(try_opt!(duration.as_u64(),
                                            ErrorStatus::InvalidArgument,
                                            "Parameter 'duration' was not a positive integer")),
            None => None

        };

        let element = match body.find("element") {
            Some(elem) => Some(try!(WebElement::from_json(elem))),
            None => None
        };

        let x = match body.find("x") {
            Some(x) => {
                Some(try_opt!(x.as_u64(),
                              ErrorStatus::InvalidArgument,
                              "Parameter 'x' was not a positive integer"))
            },
            None => None
        };

        let y = match body.find("y") {
            Some(y) => {
                Some(try_opt!(y.as_u64(),
                              ErrorStatus::InvalidArgument,
                              "Parameter 'y' was not a positive integer"))
            },
            None => None
        };

        Ok(PointerMoveAction {
            duration: duration.into(),
            element: element.into(),
            x: x.into(),
            y: y.into(),
        })
    }
}

impl ToJson for PointerMoveAction {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("type".to_owned(), "pointerMove".to_json());
        if self.duration.is_value() {
            data.insert("duration".to_owned(),
                        self.duration.to_json());
        }
        if self.element.is_value() {
            data.insert("element".to_owned(),
                        self.element.to_json());
        }
        if self.x.is_value() {
            data.insert("x".to_owned(), self.x.to_json());
        }
        if self.y.is_value() {
            data.insert("y".to_owned(), self.y.to_json());
        }
        Json::Object(data)
    }
}
