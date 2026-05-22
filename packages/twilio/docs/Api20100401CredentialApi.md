# \Api20100401CredentialApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_sip_credential**](Api20100401CredentialApi.md#create_sip_credential) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists/{CredentialListSid}/Credentials.json | Create a new credential resource.
[**delete_sip_credential**](Api20100401CredentialApi.md#delete_sip_credential) | **DELETE** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists/{CredentialListSid}/Credentials/{Sid}.json | Delete a credential resource.
[**fetch_sip_credential**](Api20100401CredentialApi.md#fetch_sip_credential) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists/{CredentialListSid}/Credentials/{Sid}.json | Fetch a single credential.
[**list_sip_credential**](Api20100401CredentialApi.md#list_sip_credential) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists/{CredentialListSid}/Credentials.json | Retrieve a list of credentials.
[**update_sip_credential**](Api20100401CredentialApi.md#update_sip_credential) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists/{CredentialListSid}/Credentials/{Sid}.json | Update a credential resource.



## create_sip_credential

> models::ApiV2010AccountSipSipCredentialListSipCredential create_sip_credential(account_sid, credential_list_sid, username, password)
Create a new credential resource.

Create a new credential resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**credential_list_sid** | **String** | The unique id that identifies the credential list to include the created credential. | [required] |
**username** | **String** | The username that will be passed when authenticating SIP requests. The username should be sent in response to Twilio's challenge of the initial INVITE. It can be up to 32 characters long. | [required] |
**password** | **String** | The password that the username will use when authenticating SIP requests. The password must be a minimum of 12 characters, contain at least 1 digit, and have mixed case. (eg `IWasAtSignal2018`) | [required] |

### Return type

[**models::ApiV2010AccountSipSipCredentialListSipCredential**](api.v2010.account.sip.sip_credential_list.sip_credential.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_sip_credential

> delete_sip_credential(account_sid, credential_list_sid, sid)
Delete a credential resource.

Delete a credential resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**credential_list_sid** | **String** | The unique id that identifies the credential list that contains the desired credentials. | [required] |
**sid** | **String** | The unique id that identifies the resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_sip_credential

> models::ApiV2010AccountSipSipCredentialListSipCredential fetch_sip_credential(account_sid, credential_list_sid, sid)
Fetch a single credential.

Fetch a single credential.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**credential_list_sid** | **String** | The unique id that identifies the credential list that contains the desired credential. | [required] |
**sid** | **String** | The unique id that identifies the resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountSipSipCredentialListSipCredential**](api.v2010.account.sip.sip_credential_list.sip_credential.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_sip_credential

> models::ListSipCredentialResponse list_sip_credential(account_sid, credential_list_sid, page_size, page, page_token)
Retrieve a list of credentials.

Retrieve a list of credentials.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**credential_list_sid** | **String** | The unique id that identifies the credential list that contains the desired credentials. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListSipCredentialResponse**](ListSipCredentialResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_sip_credential

> models::ApiV2010AccountSipSipCredentialListSipCredential update_sip_credential(account_sid, credential_list_sid, sid, password)
Update a credential resource.

Update a credential resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**credential_list_sid** | **String** | The unique id that identifies the credential list that includes this credential. | [required] |
**sid** | **String** | The unique id that identifies the resource to update. | [required] |
**password** | Option<**String**> | The password that the username will use when authenticating SIP requests. The password must be a minimum of 12 characters, contain at least 1 digit, and have mixed case. (eg `IWasAtSignal2018`) |  |

### Return type

[**models::ApiV2010AccountSipSipCredentialListSipCredential**](api.v2010.account.sip.sip_credential_list.sip_credential.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

