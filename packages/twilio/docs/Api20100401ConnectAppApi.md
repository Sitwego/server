# \Api20100401ConnectAppApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_connect_app**](Api20100401ConnectAppApi.md#delete_connect_app) | **DELETE** /2010-04-01/Accounts/{AccountSid}/ConnectApps/{Sid}.json | Delete an instance of a connect-app
[**fetch_connect_app**](Api20100401ConnectAppApi.md#fetch_connect_app) | **GET** /2010-04-01/Accounts/{AccountSid}/ConnectApps/{Sid}.json | Fetch an instance of a connect-app
[**list_connect_app**](Api20100401ConnectAppApi.md#list_connect_app) | **GET** /2010-04-01/Accounts/{AccountSid}/ConnectApps.json | Retrieve a list of connect-apps belonging to the account used to make the request
[**update_connect_app**](Api20100401ConnectAppApi.md#update_connect_app) | **POST** /2010-04-01/Accounts/{AccountSid}/ConnectApps/{Sid}.json | Update a connect-app with the specified parameters



## delete_connect_app

> delete_connect_app(account_sid, sid)
Delete an instance of a connect-app

Delete an instance of a connect-app

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the ConnectApp resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the ConnectApp resource to fetch. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_connect_app

> models::ApiV2010AccountConnectApp fetch_connect_app(account_sid, sid)
Fetch an instance of a connect-app

Fetch an instance of a connect-app

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the ConnectApp resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the ConnectApp resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountConnectApp**](api.v2010.account.connect_app.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_connect_app

> models::ListConnectAppResponse list_connect_app(account_sid, page_size, page, page_token)
Retrieve a list of connect-apps belonging to the account used to make the request

Retrieve a list of connect-apps belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the ConnectApp resources to read. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListConnectAppResponse**](ListConnectAppResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_connect_app

> models::ApiV2010AccountConnectApp update_connect_app(account_sid, sid, authorize_redirect_url, company_name, deauthorize_callback_method, deauthorize_callback_url, description, friendly_name, homepage_url, permissions)
Update a connect-app with the specified parameters

Update a connect-app with the specified parameters

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the ConnectApp resources to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the ConnectApp resource to update. | [required] |
**authorize_redirect_url** | Option<**String**> | The URL to redirect the user to after we authenticate the user and obtain authorization to access the Connect App. |  |
**company_name** | Option<**String**> | The company name to set for the Connect App. |  |
**deauthorize_callback_method** | Option<**String**> | The HTTP method to use when calling `deauthorize_callback_url`. |  |
**deauthorize_callback_url** | Option<**String**> | The URL to call using the `deauthorize_callback_method` to de-authorize the Connect App. |  |
**description** | Option<**String**> | A description of the Connect App. |  |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the resource. It can be up to 64 characters long. |  |
**homepage_url** | Option<**String**> | A public URL where users can obtain more information about this Connect App. |  |
**permissions** | Option<[**Vec<models::ConnectAppEnumPermission>**](Models__ConnectAppEnumPermission.md)> | A comma-separated list of the permissions you will request from the users of this ConnectApp.  Can include: `get-all` and `post-all`. |  |

### Return type

[**models::ApiV2010AccountConnectApp**](api.v2010.account.connect_app.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

