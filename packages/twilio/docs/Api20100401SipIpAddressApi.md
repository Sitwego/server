# \Api20100401SipIpAddressApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_sip_ip_address**](Api20100401SipIpAddressApi.md#create_sip_ip_address) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists/{IpAccessControlListSid}/IpAddresses.json | Create a new IpAddress resource.
[**delete_sip_ip_address**](Api20100401SipIpAddressApi.md#delete_sip_ip_address) | **DELETE** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists/{IpAccessControlListSid}/IpAddresses/{Sid}.json | Delete an IpAddress resource.
[**fetch_sip_ip_address**](Api20100401SipIpAddressApi.md#fetch_sip_ip_address) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists/{IpAccessControlListSid}/IpAddresses/{Sid}.json | Read one IpAddress resource.
[**list_sip_ip_address**](Api20100401SipIpAddressApi.md#list_sip_ip_address) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists/{IpAccessControlListSid}/IpAddresses.json | Read multiple IpAddress resources.
[**update_sip_ip_address**](Api20100401SipIpAddressApi.md#update_sip_ip_address) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists/{IpAccessControlListSid}/IpAddresses/{Sid}.json | Update an IpAddress resource.



## create_sip_ip_address

> models::ApiV2010AccountSipSipIpAccessControlListSipIpAddress create_sip_ip_address(account_sid, ip_access_control_list_sid, friendly_name, ip_address, cidr_prefix_length)
Create a new IpAddress resource.

Create a new IpAddress resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**ip_access_control_list_sid** | **String** | The IpAccessControlList Sid with which to associate the created IpAddress resource. | [required] |
**friendly_name** | **String** | A human readable descriptive text for this resource, up to 255 characters long. | [required] |
**ip_address** | **String** | An IP address in dotted decimal notation from which you want to accept traffic. Any SIP requests from this IP address will be allowed by Twilio. IPv4 only supported today. | [required] |
**cidr_prefix_length** | Option<**i32**> | An integer representing the length of the CIDR prefix to use with this IP address when accepting traffic. By default the entire IP address is used. |  |

### Return type

[**models::ApiV2010AccountSipSipIpAccessControlListSipIpAddress**](api.v2010.account.sip.sip_ip_access_control_list.sip_ip_address.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_sip_ip_address

> delete_sip_ip_address(account_sid, ip_access_control_list_sid, sid)
Delete an IpAddress resource.

Delete an IpAddress resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**ip_access_control_list_sid** | **String** | The IpAccessControlList Sid that identifies the IpAddress resources to delete. | [required] |
**sid** | **String** | A 34 character string that uniquely identifies the resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_sip_ip_address

> models::ApiV2010AccountSipSipIpAccessControlListSipIpAddress fetch_sip_ip_address(account_sid, ip_access_control_list_sid, sid)
Read one IpAddress resource.

Read one IpAddress resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**ip_access_control_list_sid** | **String** | The IpAccessControlList Sid that identifies the IpAddress resources to fetch. | [required] |
**sid** | **String** | A 34 character string that uniquely identifies the IpAddress resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountSipSipIpAccessControlListSipIpAddress**](api.v2010.account.sip.sip_ip_access_control_list.sip_ip_address.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_sip_ip_address

> models::ListSipIpAddressResponse list_sip_ip_address(account_sid, ip_access_control_list_sid, page_size, page, page_token)
Read multiple IpAddress resources.

Read multiple IpAddress resources.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**ip_access_control_list_sid** | **String** | The IpAccessControlList Sid that identifies the IpAddress resources to read. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListSipIpAddressResponse**](ListSipIpAddressResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_sip_ip_address

> models::ApiV2010AccountSipSipIpAccessControlListSipIpAddress update_sip_ip_address(account_sid, ip_access_control_list_sid, sid, ip_address, friendly_name, cidr_prefix_length)
Update an IpAddress resource.

Update an IpAddress resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**ip_access_control_list_sid** | **String** | The IpAccessControlList Sid that identifies the IpAddress resources to update. | [required] |
**sid** | **String** | A 34 character string that identifies the IpAddress resource to update. | [required] |
**ip_address** | Option<**String**> | An IP address in dotted decimal notation from which you want to accept traffic. Any SIP requests from this IP address will be allowed by Twilio. IPv4 only supported today. |  |
**friendly_name** | Option<**String**> | A human readable descriptive text for this resource, up to 255 characters long. |  |
**cidr_prefix_length** | Option<**i32**> | An integer representing the length of the CIDR prefix to use with this IP address when accepting traffic. By default the entire IP address is used. |  |

### Return type

[**models::ApiV2010AccountSipSipIpAccessControlListSipIpAddress**](api.v2010.account.sip.sip_ip_access_control_list.sip_ip_address.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

